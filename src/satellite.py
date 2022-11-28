from os import environ, _exit
environ["PRODUCTION"] = environ["PRODUCTION"] if "PRODUCTION" in environ and environ["PRODUCTION"] else ""
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

from helpers import constants
from DatabaseConnector import DatabaseConnector
from Processor import process_quote_arguments, process_task


database = FirestoreAsyncClient()
logging = ErrorReportingClient(service="satellites")


# -------------------------
# Initialization
# -------------------------

intents = Intents.none()
intents.guilds = True

bot = AutoShardedClient(intents=intents, chunk_guilds_at_startup=False, max_messages=None, status=Status.idle, activity=None)


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

if platform == "CCXT": refreshRate = 2.0
else: refreshRate = 5.0


# -------------------------
# Guild events
# -------------------------

@bot.event
async def on_guild_join(guild):
	try:
		if isFree: return

		properties = await guildProperties.get(guild.id)
		if properties is None: return

		if properties['settings']['setup']['connection'] is not None:
			await database.document(f"accounts/{properties['settings']['setup']['connection']}").set({"customer": {"slots": {"satellites": {str(guild.id): {"added": ArrayUnion([str(bot.user.id)])}}}}}, merge=True)

		await update_properties()
	except Exception:
		print(format_exc())
		if environ["PRODUCTION"]: logging.report_exception(user=str(guild.id))

@bot.event
async def on_guild_remove(guild):
	try:
		if isFree: return

		properties = await guildProperties.get(guild.id)
		if properties is None: return

		if properties['settings']['setup']['connection'] is not None:
			await database.document(f"accounts/{properties['settings']['setup']['connection']}").set({"customer": {"slots": {"satellites": {str(guild.id): {"added": ArrayRemove([str(bot.user.id)])}}}}}, merge=True)

		await update_properties()
	except Exception:
		print(format_exc())
		if environ["PRODUCTION"]: logging.report_exception(user=str(guild.id))


# -------------------------
# Tasks
# -------------------------

@tasks.loop(minutes=60.0)
async def update_properties():
	try:
		if priceText is not None:
			guildIds = [str(e.id) for e in bot.guilds]
			await database.document(f"satellites/{bot.user.id}").set({
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
		if environ["PRODUCTION"]: logging.report_exception()

@tasks.loop(minutes=60.0)
async def update_ticker():
	global request
	try:
		responseMessage, request = await process_quote_arguments([] if exchange is None else [exchange], [platform], tickerId=tickerId)
		if responseMessage is not None:
			print("Parsing failed:", responseMessage)
			print(request)
			request = None
			return False

		request["bot"] = True
		return True
	except CancelledError: return
	except Exception:
		print(format_exc())
		if environ["PRODUCTION"]: logging.report_exception()

@tasks.loop(minutes=refreshRate)
async def update_nicknames():
	global updatingNickname, priceText, statusText
	try:
		updatingNickname = True
		await sleep(timeOffset)

		if request is None or len(request.get("platforms", [])) == 0:
			success = await update_ticker()
			if not success: return

		try: payload, responseMessage = await process_task(request, "quote", retries=1)
		except: return
		if payload is None or "quotePrice" not in payload:
			print("Something went wrong when fetching the price:", bot.user.id, responseMessage)
			print(payload)
			return

		currentRequest = request.get(payload.get("platform"))
		ticker = currentRequest.get("ticker")
		exchangeId = ticker.get("exchange", {}).get("id")

		priceText = payload["quotePrice"]
		changeText = f"{payload['change']} | " if "change" in payload else ""
		tickerText = f"{ticker.get('id')} on {ticker.get('exchange').get('name')} | " if exchangeId is not None and exchangeId != "forex" else f"{ticker.get('id')} | "
		statusText = f"{changeText}{tickerText}www.alpha.bot"
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

				# Get all filled satellite slots
				slots = properties.get("connection", {}).get("customer", {}).get("slots", {}).get("satellites", {})
				# Get subscription quantity
				subscription = properties.get("connection", {}).get("customer", {}).get("subscriptions", {}).get("satellites", 0)

				# Empty list by default when subscription doesn't cover servers up to current one
				added = []
				# Iterate over sorted guilds in satellite slot configuration
				for guildId in sorted(slots.keys()):
					# Get all bots added to each server
					bots = slots[guildId].get("added", [])
					if subscription <= 0:
						# If slots run out, break out resulting in added == []
						break
					if guildId == str(guild.id):
						# If we get to the current guild, get a sorted list of added bots capped at the available slot count
						added = sorted(bots)[:subscription]
						# Stop the search
						break
					else:
						# Subtract used slots from total slots
						subscription -= len(bots)

				if str(bot.user.id) not in slots.get(str(guild.id), {}).get("added", []):
					# If bot isn't added to the list of all bots in the server, add it
					if properties['settings']['setup']['connection'] is not None:
						await database.document(f"accounts/{properties['settings']['setup']['connection']}").set({"customer": {"slots": {"satellites": {str(guild.id): {"added": ArrayUnion([str(bot.user.id)])}}}}}, merge=True)
					# Continue, otherwise it would show as "One more subscription slot required"
					continue

				if str(bot.user.id) in added:
					await update_nickname(guild, priceText)
				else:
					await update_nickname(guild, "One more subscription slot required")

		try: await bot.change_presence(status=status, activity=Activity(type=ActivityType.watching, name=statusText))
		except: pass

	except CancelledError: return
	except Exception:
		print(format_exc())
		if environ["PRODUCTION"]: logging.report_exception()
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

@bot.event
async def on_ready():
	await sleep(randint(0, int(refreshRate * 60)))
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