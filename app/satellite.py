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

priceText = None
statusText = None

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
async def on_guild_join(guild):
	try:
		await update_properties()
	except Exception:
		print(format_exc())
		if environ["PRODUCTION_MODE"]: logging.report_exception(user=str(guild.id))

@bot.event
async def on_guild_remove(guild):
	try:
		properties = await guildProperties.get(guild.id)
		if properties is None: return

		if str(bot.user.id) in properties["addons"]["satellites"].get("added", []):
			await database.document(f"discord/properties/guilds/{guild.id}").set({"addons": {"satellites": {"added": ArrayRemove([str(bot.user.id)])}}}, merge=True)

		await update_properties()
	except Exception:
		print(format_exc())
		if environ["PRODUCTION_MODE"]: logging.report_exception(user=str(guild.id))


# -------------------------
# Tasks
# -------------------------

@tasks.loop(minutes=60.0)
async def update_properties():
	try:
		if priceText is not None:
			guildIds = [str(e.id) for e in bot.guilds]
			await database.document(f"dataserver/configuration/satellites/{bot.user.id}").set({
				"count": len(guildIds),
				"servers": guildIds,
				"user": {
					"icon": str(bot.user.avatar.replace(format="png", size=512)),
					"name": bot.user.name,
					"watching": statusText,
					"price": priceText
				}
			})
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
	global updatingNickname, priceText, statusText
	try:
		updatingNickname = True
		await sleep(timeOffset)

		if request is None or len(request.get("platforms", [])) == 0:
			success = await update_ticker(force=True)
			if not success: return

		try: payload, quoteText = await Processor.process_task("quote", bot.user.id, request)
		except: return
		if payload is None or "quotePrice" not in payload:
			print("Something went wrong when fetching the price:", bot.user.id, quoteText)
			print(payload)
			return

		currentRequest = request.get(payload.get("platform"))
		ticker = currentRequest.get("ticker")

		priceText = payload["quotePrice"]
		changeText = f"{payload['change']} | " if "change" in payload else ""
		tickerText = f"{ticker.get('id')} on {ticker.get('exchange').get('name')} | " if ticker.get("exchange") else f"{ticker.get('id')} | "
		statusText = f"{changeText}{tickerText}alphabotsystem.com"
		status = Status.dnd if payload.get("messageColor") == "red" else Status.online

		for guild in bot.guilds:
			if not guild.me.guild_permissions.change_nickname:
				continue

			if isFree:
				await update_nickname(guild, priceText)

			else:
				properties = await guildProperties.get(guild.id)
				if properties is None:
					await sleep(0.5)
					continue

				slots = properties.get("connection", {}).get("customer", {}).get("slots", {}).get("satellites", {}).get(guild.id, 0)
				added = sorted(properties["addons"]["satellites"].get("added", []))

				if str(bot.user.id) not in added:
					await database.document(f"discord/properties/guilds/{guild.id}").set({"addons": {"satellites": {"added": ArrayUnion([str(bot.user.id)])}}}, merge=True)
					added.append(str(bot.user.id))
    
				if str(bot.user.id) in added[:slots] or True: # Temp
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
	if not update_properties.is_running():
		update_properties.start()
	if not update_ticker.is_running():
		update_ticker.start()
	if not update_nicknames.is_running():
		update_nicknames.start()

	print("[Startup]: Alpha Satellite is online")


# -------------------------
# Login
# -------------------------

token = environ[f"ID_{constants.satellites[satelliteId]}"]
bot.run(token)