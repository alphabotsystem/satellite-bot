from os import environ, _exit
environ["PRODUCTION_MODE"] = environ["PRODUCTION_MODE"] if "PRODUCTION_MODE" in environ and environ["PRODUCTION_MODE"] else ""
satelliteId = 0 if len(environ["HOSTNAME"].split("-")) == 1 else int(environ["HOSTNAME"].split("-")[-1])
if not environ.get("IS_FREE"): satelliteId += 2

from time import time
from random import randint
from datetime import datetime
from pytz import utc
from asyncio import CancelledError, sleep
from traceback import format_exc

from discord import AutoShardedClient, Embed, Intents, Activity, Status, ActivityType
from discord.ext import tasks
from google.cloud.firestore import AsyncClient as FirestoreAsyncClient
from google.cloud.firestore import ArrayUnion, ArrayRemove
from google.cloud.error_reporting import Client as ErrorReportingClient

from helpers.utils import Utils
from helpers import constants

from DatabaseConnector import DatabaseConnector
from Processor import Processor

from CommandRequest import CommandRequest


database = FirestoreAsyncClient()
logging = ErrorReportingClient(service="satellites")


# -------------------------
# Initialization
# -------------------------

intents = Intents.none()
intents.guilds = True

bot = AutoShardedClient(intents=intents, status=Status.idle, activity=None)


# -------------------------
# Task setup
# -------------------------

request = None
updatingNickname = False
timeOffset = randint(0, 600) / 10.0
platform, exchange, tickerId = constants.configuration[constants.satellites[satelliteId]]
isFree = platform == "CoinGecko" and exchange is None and tickerId in ["BTCUSD", "ETHUSD"]

if platform == "CCXT": refreshRate = 1.0
else: refreshRate = 5.0


# -------------------------
# Guild events
# -------------------------

@bot.event
async def on_guild_remove(guild):
	try:
		properties = await guildProperties.get(guild.id)
		if properties is None: return

		if str(bot.user.id) in properties["addons"]["satellites"].get("added", []):
			await database.document("discord/properties/guilds/{}".format(guild.id)).set({"addons": {"satellites": {"added": ArrayRemove([str(bot.user.id)])}}}, merge=True)
	except Exception:
		print(format_exc())
		if environ["PRODUCTION_MODE"]: logging.report_exception(user=str(guild.id))


# -------------------------
# Job functions
# -------------------------

@tasks.loop(minutes=60.0)
async def update_properties():
	try:
		satelliteRef = database.document("dataserver/configuration/satellites/{}".format(bot.user.id))
		properties = await satelliteRef.get()
		properties = properties.to_dict()

		guildIds = [str(e.id) for e in bot.guilds]
		for guildId in properties.get("servers", []):
			if guildId not in guildIds:
				await database.document("discord/properties/guilds/{}".format(guildId)).set({"addons": {"satellites": {"added": ArrayRemove([guildId])}}}, merge=True)

		await satelliteRef.set({"count": len(guildIds), "servers": guildIds})
	except CancelledError: return
	except Exception:
		print(format_exc())
		if environ["PRODUCTION_MODE"]: logging.report_exception()

@tasks.loop(minutes=60.0)
async def update_ticker(force=False):
	global request
	try:
		if not force:
			# Make the request at random in order not to stress the parsing server too much
			await sleep(randint(0, 3600))

		outputMessage, request = await Processor.process_quote_arguments(CommandRequest(), [] if exchange is None else [exchange], [platform], tickerId=tickerId)
		if outputMessage is not None:
			print("Parsing failed:", outputMessage)
			print(request)
			return False
		return True
	except CancelledError: return
	except Exception:
		print(format_exc())
		if environ["PRODUCTION_MODE"]: logging.report_exception()

@tasks.loop(minutes=refreshRate)
async def update_nicknames():
	global updatingNickname
	try:
		updatingNickname = True
		await sleep(timeOffset)

		if request is None or len(request.get("platforms", [])) == 0:
			success = await update_ticker(force=True)
			if not success: return

		try: payload, quoteText = await Processor.process_task("quote", bot.user.id, request)
		except: pass
		if payload is None or "quotePrice" not in payload:
			print("Something went wrong when fetching the price:", bot.user.id, quoteText)
			print(payload)
			return

		currentRequest = request.get(payload.get("platform"))
		ticker = currentRequest.get("ticker")

		priceText = payload["quotePrice"]
		changeText = "{} | ".format(payload["change"]) if "change" in payload else ""
		tickerText = "{} | ".format(ticker.get("id")) if not bool(ticker.get("exchange")) else "{} on {} | ".format(ticker.get("id"), ticker.get("exchange").get("name"))
		statusText = "{}{}alphabotsystem.com".format(changeText, tickerText)
		status = Status.dnd if payload.get("messageColor") == "red" else Status.online

		for guild in bot.guilds:
			if not guild.me.guild_permissions.change_nickname:
				continue

			if isFree:
				await update_nickname(guild, priceText)

			else:
				_guildProperties = await guildProperties.get(guild.id)
				if _guildProperties is None:
					await sleep(0.5)
					continue

				connection = _guildProperties.get("addons", {}).get("satellites", {}).get("connection", _guildProperties.get("settings", {}).get("setup", {}).get("connection"))
				_accountProperties = await accountProperties.get(connection)
				if _accountProperties is None:
					await sleep(0.5)
					continue

				if _accountProperties.get("customer", {}).get("personalSubscription", {}).get("subscription") is not None:
					if not _guildProperties["addons"]["satellites"]["enabled"]:
						await database.document("discord/properties/guilds/{}".format(guild.id)).set({"addons": {"satellites": {"enabled": True, "connection": connection}}}, merge=True)
					if str(bot.user.id) not in _guildProperties["addons"]["satellites"].get("added", []):
						await database.document("discord/properties/guilds/{}".format(guild.id)).set({"addons": {"satellites": {"added": ArrayUnion([str(bot.user.id)])}}}, merge=True)
					await update_nickname(guild, priceText)
				else:
					await update_nickname(guild, "Alpha Pro required")

		try: await bot.change_presence(status=status, activity=Activity(type=ActivityType.watching, name=statusText))
		except: pass

	except CancelledError: return
	except Exception:
		print(format_exc())
		if environ["PRODUCTION_MODE"]: logging.report_exception()
	finally:
		updatingNickname = False

async def update_nickname(guild, nickname):
	if guild.me.nick != nickname:
		try: await guild.me.edit(nick=nickname)
		except: pass
	else:
		await sleep(0.5)


# -------------------------
# Startup
# -------------------------

accountProperties = DatabaseConnector(mode="account")
guildProperties = DatabaseConnector(mode="guild")
Processor.clientId = b"discord_satellite"

@bot.event
async def on_ready():
	update_properties.start()
	update_ticker.start()
	update_nicknames.start()

	print("[Startup]: Alpha Satellite is online")


# -------------------------
# Login
# -------------------------

token = environ["ID_{}".format(constants.satellites[satelliteId])]
bot.run(token)