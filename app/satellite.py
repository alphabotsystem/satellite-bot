from os import environ
from random import randint
from time import time
from datetime import datetime
from uuid import uuid4
from pytz import utc
from asyncio import CancelledError, InvalidStateError, TimeoutError, sleep, all_tasks, wait_for
from traceback import format_exc

import discord
from google.cloud.firestore import AsyncClient as FirestoreAsnycClient
from google.cloud.firestore import ArrayUnion, ArrayRemove
from google.cloud.error_reporting import Client as ErrorReportingClient

from Processor import Processor
from DatabaseConnector import DatabaseConnector

from MessageRequest import MessageRequest
from helpers.utils import Utils
from helpers import constants


database = FirestoreAsnycClient()


class Alpha(discord.AutoShardedClient):
	isBotReady = False
	updatingNickname = False
	timeOffset = 0

	accountProperties = DatabaseConnector(mode="account")
	guildProperties = DatabaseConnector(mode="guild")

	tickerId = None
	exchange = None
	platform = None
	isFree = False


	def prepare(self):
		Processor.clientId = b"discord_satellite"
		self.logging = ErrorReportingClient(service="satellites")
		self.timeOffset = randint(0, 600) / 10.0
		self.priceText = None

	async def on_ready(self):
		self.platform, self.exchange, self.tickerId = constants.configuration[client.user.id]
		self.isFree = self.platform == "CoinGecko" and self.exchange is None and self.tickerId in ["BTCUSD", "ETHUSD"]

		self.isBotReady = True
		print("[Startup]: Alpha Satellite is online")

	async def on_guild_remove(self, guild):
		try:
			guildProperties = await self.guildProperties.get(guild.id)
			if guildProperties is None: return

			if str(client.user.id) in guildProperties["addons"]["satellites"].get("added", []):
				await database.document("discord/properties/guilds/{}".format(guild.id)).set({"addons": {"satellites": {"added": ArrayRemove([str(client.user.id)])}}}, merge=True)
		except Exception:
			print(format_exc())
			if environ["PRODUCTION_MODE"]: self.logging.report_exception(user=str(guild.id))

	async def job_queue(self):
		while True:
			try:
				await sleep(Utils.seconds_until_cycle())
				t = datetime.now().astimezone(utc)
				timeframes = Utils.get_accepted_timeframes(t)

				isPremium = self.tickerId in ["EURUSD", "GBPUSD", "AUDJPY", "AUDUSD", "EURJPY", "GBPJPY", "NZDJPY", "NZDUSD", "CADUSD", "JPYUSD", "ZARUSD"]
				if len(client.guilds) == 1: refreshRate = "8H"
				elif isPremium and len(client.guilds) < 15: refreshRate = "1H"
				elif self.platform == "CCXT": refreshRate = "1m"
				else: refreshRate = "5m"

				if refreshRate in timeframes and not self.updatingNickname:
					client.loop.create_task(self.update_nicknames())
				if "1H" in timeframes:
					client.loop.create_task(self.update_properties())

			except CancelledError: return
			except Exception:
				print(format_exc())
				if environ["PRODUCTION_MODE"]: self.logging.report_exception()

	async def update_properties(self):
		try:
			satelliteRef = database.document("dataserver/configuration/satellites/{}".format(client.user.id))
			properties = await satelliteRef.get()
			properties = properties.to_dict()

			guildIds = [str(e.id) for e in client.guilds]
			for guildId in properties.get("servers", []):
				if guildId not in guildIds:
					await database.document("discord/properties/guilds/{}".format(guildId)).set({"addons": {"satellites": {"added": ArrayRemove([guildId])}}}, merge=True)

			await satelliteRef.set({"count": len(guildIds), "servers": guildIds})
		except CancelledError: return
		except Exception:
			print(format_exc())
			if environ["PRODUCTION_MODE"]: self.logging.report_exception()

	async def update_nicknames(self):
		try:
			self.updatingNickname = True
			await sleep(self.timeOffset)

			outputMessage, request = await Processor.process_quote_arguments(MessageRequest(), [] if self.exchange is None else [self.exchange], tickerId=self.tickerId, platformQueue=[self.platform])
			if outputMessage is not None:
				print("Parsing failed:", outputMessage)
				print(request)
				return

			try: payload, quoteText = await Processor.process_request("quote", client.user.id, request)
			except: return
			if payload is None or "quotePrice" not in payload:
				print("Something wen't wrong when fetching the price:", quoteText)
				print(payload)
				return

			currentRequest = request.get(payload.get("platform"))
			ticker = currentRequest.get("ticker")

			self.priceText = payload["quotePrice"]
			changeText = "{} | ".format(payload["change"]) if "change" in payload else ""
			tickerText = "{} | ".format(ticker.get("id")) if not bool(ticker.get("exchange")) else "{} on {} | ".format(ticker.get("id"), ticker.get("exchange").get("name"))
			statusText = "{}{}alphabotsystem.com".format(changeText, tickerText)
			status = discord.Status.dnd if payload.get("messageColor") == "red" else discord.Status.online

			for guild in client.guilds:
				if not guild.me.guild_permissions.change_nickname:
					continue

				if self.isFree:
					await self.update_nickname(guild, self.priceText)

				else:
					guildProperties = await self.guildProperties.get(guild.id)
					if guildProperties is None:
						await sleep(0.5)
						continue

					connection = guildProperties.get("addons", {}).get("satellites", {}).get("connection", guildProperties.get("settings", {}).get("setup", {}).get("connection"))
					accountProperties = await self.accountProperties.get(connection)
					if accountProperties is None:
						await sleep(0.5)
						continue

					if accountProperties.get("customer", {}).get("personalSubscription", {}).get("subscription") is not None:
						if not guildProperties["addons"]["satellites"]["enabled"]:
							await database.document("discord/properties/guilds/{}".format(guild.id)).set({"addons": {"satellites": {"enabled": True, "connection": connection}}}, merge=True)
						if str(client.user.id) not in guildProperties["addons"]["satellites"].get("added", []):
							await database.document("discord/properties/guilds/{}".format(guild.id)).set({"addons": {"satellites": {"added": ArrayUnion([str(client.user.id)])}}}, merge=True)
						await self.update_nickname(guild, self.priceText)
					else:
						await self.update_nickname(guild, "Alpha Pro required")

			try: await client.change_presence(status=status, activity=discord.Activity(type=discord.ActivityType.watching, name=statusText))
			except: pass

		except CancelledError: return
		except Exception:
			print(format_exc())
			if environ["PRODUCTION_MODE"]: self.logging.report_exception()
		finally:
			self.updatingNickname = False

	async def update_nickname(self, guild, nickname):
		if guild.me.nick != nickname:
			try: await guild.me.edit(nick=nickname)
			except: pass
		else:
			await sleep(0.5)


# -------------------------
# Initialization
# -------------------------

def handle_exit(sleepDuration=0):
	print("\n[Shutdown]: closing tasks")
	try: client.loop.run_until_complete(client.close())
	except: pass
	for t in all_tasks(loop=client.loop):
		if t.done():
			try: t.exception()
			except InvalidStateError: pass
			except TimeoutError: pass
			except CancelledError: pass
			continue
		t.cancel()
		try:
			client.loop.run_until_complete(wait_for(t, 5, loop=client.loop))
			t.exception()
		except InvalidStateError: pass
		except TimeoutError: pass
		except CancelledError: pass
	from time import sleep as ssleep
	ssleep(sleepDuration)

if __name__ == "__main__":
	environ["PRODUCTION_MODE"] = environ["PRODUCTION_MODE"] if "PRODUCTION_MODE" in environ and environ["PRODUCTION_MODE"] else ""
	print("[Startup]: Alpha Satellite is in startup, running in {} mode.".format("production" if environ["PRODUCTION_MODE"] else "development"))
	satelliteId = 0 if len(environ["HOSTNAME"].split("-")) == 1 else int(environ["HOSTNAME"].split("-")[-1])
	if not environ.get("IS_FREE"): satelliteId += 2

	intents = discord.Intents.none()
	intents.guilds = True

	client = Alpha(intents=intents, status=discord.Status.idle, activity=None)
	print("[Startup]: object initialization complete")
	client.prepare()

	while True:
		client.loop.create_task(client.job_queue())
		try:
			token = environ["ID_{}".format(constants.satellites[satelliteId])]
			client.loop.run_until_complete(client.start(token))
		except (KeyboardInterrupt, SystemExit):
			handle_exit()
			client.loop.close()
			break
		except Exception:
			print(format_exc())
			handle_exit(sleepDuration=900)

		client = Alpha(loop=client.loop, intents=intents, status=discord.Status.idle, activity=None)