mod config;
use config::{
    CONFIGURATION, FREE_THRESHOLD, PRICE_REFRESH_SECONDS, PROJECT, REQUEST_REFRESH_SECONDS,
    SATELLITES,
};
use database::{DatabaseConnector, GuildProperties};
use firestore::*;
use processor::{process_quote_arguments, process_task};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{Guild, OnlineStatus, UnavailableGuild, UserId};
use serenity::async_trait;
use serenity::gateway::ActivityData;
use serenity::model::{gateway::Ready, id::GuildId};
use serenity::prelude::*;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UserInfo {
    pub icon: String,
    pub name: String,
    pub status: String,
    pub price: String,
}
impl TypeMapKey for UserInfo {
    type Value = Arc<RwLock<Option<UserInfo>>>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SatelliteProperties {
    pub count: usize,
    pub servers: Vec<String>,
    pub user: UserInfo,
}

struct RequestCache;
impl TypeMapKey for RequestCache {
    type Value = Arc<RwLock<Option<Value>>>;
}

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
		_ctx.set_presence(None, OnlineStatus::Idle);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            self.is_loop_running.swap(true, Ordering::Relaxed);

            let ctx = Arc::new(_ctx);
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    update_ticker(Arc::clone(&ctx1)).await;
                    update_properties(Arc::clone(&ctx1)).await;
                    sleep(Duration::from_secs(REQUEST_REFRESH_SECONDS)).await;
                }
            });

            sleep(Duration::from_secs(5)).await;

            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    let duration = update_nicknames(Arc::clone(&ctx2)).await;
                    if duration.as_secs() < PRICE_REFRESH_SECONDS {
                        sleep(Duration::from_secs(PRICE_REFRESH_SECONDS) - duration).await;
                    }
                }
            });
        }

        println!(
            "[Startup]: Alpha.bot Satellite ({}) is online",
            ready.user.id
        );
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        if !_is_new.unwrap_or(false) {
            return;
        }

        let bot_id = _ctx.cache.current_user().id;
        let guild_id = guild.id.0.to_string();

        let guild_properties = DatabaseConnector::<GuildProperties>::new();
        let properties = match guild_properties.get(&guild_id, None).await {
            Some(properties) => properties,
            None => {
                println!("[{}]: Couldn't fetch properties for {}", bot_id, guild_id);
                return;
            }
        };

        if properties.connection.is_none() {
            return;
        }

        let database = FirestoreDb::new(PROJECT)
            .await
            .expect("Couldn't connect to Firestore");
        let mut transaction = database
            .begin_transaction()
            .await
            .expect("Couldn't start transaction");

        database
            .fluent()
            .update()
            .in_col("accounts")
            .document_id(properties.settings.setup.connection.unwrap())
            .transforms(|t| {
                let field = format!("customer.slots.satellites.`{}`.added", guild_id);
                t.fields([t
                    .field(field)
                    .append_missing_elements([bot_id.0.get().to_string()])])
            })
            .only_transform()
            .add_to_transaction(&mut transaction)
            .expect("Couldn't add update to transaction");

        transaction
            .commit()
            .await
            .expect("Couldn't commit transaction");
    }

    async fn guild_delete(&self, _ctx: Context, guild: UnavailableGuild, _full: Option<Guild>) {
        let bot_id = _ctx.cache.current_user().id;
        let guild_id = guild.id.0.to_string();

        let guild_properties = DatabaseConnector::<GuildProperties>::new();
        let properties = match guild_properties.get(&guild_id, None).await {
            Some(properties) => properties,
            None => {
                println!("[{}]: Couldn't fetch properties for {}", bot_id, guild_id);
                return;
            }
        };

        if properties.connection.is_none() {
            return;
        }

        let database = FirestoreDb::new(PROJECT)
            .await
            .expect("Couldn't connect to Firestore");
        let mut transaction = database
            .begin_transaction()
            .await
            .expect("Couldn't start transaction");

        database
            .fluent()
            .update()
            .in_col("accounts")
            .document_id(properties.settings.setup.connection.unwrap())
            .transforms(|t| {
                let field = format!("customer.slots.satellites.`{}`.added", guild_id);
                t.fields([t
                    .field(field)
                    .remove_all_from_array([bot_id.0.get().to_string()])])
            })
            .only_transform()
            .add_to_transaction(&mut transaction)
            .expect("Couldn't add update to transaction");

        transaction
            .commit()
            .await
            .expect("Couldn't commit transaction");
    }
}

async fn update_ticker(ctx: Arc<Context>) {
    let bot_id = ctx.cache.current_user().id;
    let (platform, exchange, ticker_id) = CONFIGURATION.get(&bot_id.0.get().to_string()).unwrap();

    println!(
        "[{}]: Updating cached request for {}:{}:{}",
        bot_id,
        platform,
        exchange.unwrap_or("_"),
        ticker_id
    );

    // Make parser request
    let arguments = match exchange {
        Some(exchange) => vec![*exchange],
        None => vec![],
    };
    let (message, request) =
        process_quote_arguments(arguments, vec![platform], Some(ticker_id)).await;

    if request.is_null() || !message.is_null() {
        eprintln!("[{}]: Parsing failed: {}", bot_id, message);
        eprintln!("[{}]: {:?}", bot_id, request);
        return;
    }

    // Update global cache
    let lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<RequestCache>()
            .expect("Expected RequestCache in TypeMap")
            .clone()
    };

    {
        lock.write().await.replace(request);
    }
}

async fn update_properties(ctx: Arc<Context>) {
    let bot_id = ctx.cache.current_user().id;

    // Initialize database connection
    let database = FirestoreDb::new(PROJECT)
        .await
        .expect("Couldn't connect to Firestore");

    // Obtain cached user info object
    let lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<UserInfo>()
            .expect("Expected UserInfo in TypeMap")
            .clone()
    };
    let user_info = match lock.read().await.clone() {
        Some(user_info) => user_info,
        None => {
            println!("[{}]: User info has not been cached yet", bot_id);
            return;
        }
    };

    // Update properties
    let guilds = ctx.cache.guilds();
    let properties = SatelliteProperties {
        count: guilds.len(),
        servers: guilds.iter().map(|x| x.0.get().to_string()).collect(),
        user: user_info,
    };

    // Push properties to database
    let result = database
        .fluent()
        .update()
        .in_col("satellites")
        .document_id(bot_id.0.get().to_string())
        .object(&properties)
        .execute::<()>()
        .await;

    if let Err(err) = result {
        eprintln!("[{}]: Couldn't update properties: {:?}", bot_id, err);
    }
}

async fn update_nicknames(ctx: Arc<Context>) -> Duration {
    let start = Instant::now();
    let bot_id = ctx.cache.current_user().id;
    let is_free = env::var("IS_FREE").is_ok();

    println!("[{}]: Updating nicknames", bot_id);

    // Obtain cached request object
    let request = {
        let lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<RequestCache>()
                .expect("Expected RequestCache in TypeMap")
                .clone()
        };

        let request = lock.read().await.clone();
        match request {
            Some(request) => request,
            None => {
                println!("[{}]: Request has not been cached yet", bot_id);
                return Duration::from_secs(0);
            }
        }
    };

    // Make quote request
    let response = process_task(request.clone(), "quote", None, None, None).await;
    let data = match response {
        Ok(response) => response,
        Err(err) => {
            eprintln!(
                "[{}]: Something went wrong when fetching the price: {}",
                bot_id, err
            );
            return Duration::from_secs(0);
        }
    };

    // Get all the necessary data
    let payload = data.get("response").expect("Expected response in payload");
    let message = data.get("message").expect("Expected message in payload");

    if !message.is_null() || payload.get("quotePrice").is_none() {
        eprintln!(
            "[{}]: Something went wrong when fetching the price: {}",
            bot_id, message
        );
        eprintln!("[{}]: {:?}", bot_id, payload);
        return Duration::from_secs(0);
    }

    let platform = payload
        .get("platform")
        .expect("Expected platform in payload")
        .as_str()
        .unwrap();
    let current_request = request
        .get(platform)
        .expect("Expected platform specific data in request");
    let ticker = current_request
        .get("ticker")
        .expect("Expected ticker in request");
    let exchange_id = match ticker.get("exchange") {
        Some(exchange) => exchange.get("id"),
        None => None,
    };

    // Generate all the necessary text
    let (price_text, change_text, ticker_text) = match platform {
        "Alternative.me" => (
            format!(
                "{} {}",
                payload
                    .get("quotePrice")
                    .expect("Expected quotePrice in payload")
                    .as_str()
                    .unwrap(),
                payload
                    .get("quoteConvertedPrice")
                    .expect("Expected quoteConvertedPrice in payload")
                    .as_str()
                    .unwrap()
            ),
            "crypto | ".to_string(),
            format!(
                "{} | ",
                payload
                    .get("change")
                    .expect("Expected change in payload")
                    .as_str()
                    .unwrap()
            ),
        ),
        "CNN Business" => (
            format!(
                "{} {}",
                payload
                    .get("quotePrice")
                    .expect("Expected quotePrice in payload")
                    .as_str()
                    .unwrap(),
                payload
                    .get("quoteConvertedPrice")
                    .expect("Expected quoteConvertedPrice in payload")
                    .as_str()
                    .unwrap()
            ),
            "stocks | ".to_string(),
            format!(
                "{} | ",
                payload
                    .get("change")
                    .expect("Expected change in payload")
                    .as_str()
                    .unwrap()
            ),
        ),
        _ => (
            payload
                .get("quotePrice")
                .expect("Expected quotePrice in payload")
                .as_str()
                .unwrap()
                .to_string(),
            if payload.get("change").is_some() {
                format!("{} | ", payload.get("change").unwrap().as_str().unwrap())
            } else {
                "".to_string()
            },
            if exchange_id.is_some() && exchange_id.unwrap() != "forex" {
                format!(
                    "{} on {} | ",
                    ticker
                        .get("id")
                        .expect("Expected id in ticker")
                        .as_str()
                        .unwrap(),
                    ticker
                        .get("exchange")
                        .expect("Expected exchange in ticker")
                        .get("name")
                        .expect("Expected name in exchange")
                        .as_str()
                        .unwrap()
                )
            } else {
                format!(
                    "{} | ",
                    ticker
                        .get("id")
                        .expect("Expected id in ticker")
                        .as_str()
                        .unwrap()
                )
            },
        ),
    };

    let state = format!("{}{}www.alpha.bot", change_text, ticker_text);
    let status = if payload.get("messageColor").unwrap().as_str() == Some("red") {
        OnlineStatus::DoNotDisturb
    } else {
        OnlineStatus::Online
    };

    // Update global cache
    let lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<UserInfo>()
            .expect("Expected UserInfo in TypeMap")
            .clone()
    };

    {
        lock.write().await.replace(UserInfo {
            icon: ctx
                .cache
                .current_user()
                .avatar_url()
                .unwrap_or("".to_string()),
            name: ctx.cache.current_user().name.clone(),
            status: state.clone(),
            price: price_text.clone(),
        });
    }

    // Initialize database connections
    let guild_properties = DatabaseConnector::<GuildProperties>::new();
	let database = FirestoreDb::new(PROJECT)
        .await
        .expect("Couldn't connect to Firestore");
    let mut transaction = database
        .begin_transaction()
        .await
        .expect("Couldn't start transaction");
	let mut needs_commit = false;

    // Update guild nicknames
    let guilds = ctx.cache.guilds();
    for guild in guilds.iter() {
        if is_free {
            // If the bot is in the free tier, update the nickname immediately
            update_nickname(&ctx, bot_id, guild, &price_text).await;
        } else {
            // Get guild properties
            let guild_id = guild.0.get().to_string();
            let properties = match guild_properties.get(&guild_id, None).await {
                Some(properties) => properties,
                None => {
                    println!("[{}]: Couldn't fetch properties for {}", bot_id, guild);
                    continue;
                }
            };

            let customer = match properties.connection {
                Some(connection) => connection.customer,
                None => {
                    // Looks like the bot isn't set up yet
                    update_nickname(&ctx, bot_id, guild, "Alpha.bot not set up").await;
                    continue;
                }
            };

            // Get all filled satellite slots
            let slots = customer.slots.get("satellites");
            // Get slot count
            let mut subscription = *customer.subscriptions.get("satellites").unwrap_or(&0);

            // Empty list by default when subscription doesn't cover servers up to current one
            let mut in_free_tier = false;
            let mut added: Vec<String> = vec![];

            // Iterate over sorted guilds in satellite slot configuration
            if let Some(slots) = slots {
                // Get all bots added to each server
                for (key, value) in slots.iter() {
                    let bots = match value.added.as_ref() {
                        Some(bots) => bots,
                        None => continue,
                    };
                    let bot_count = FREE_THRESHOLD.min(bots.len() as u32);

                    if subscription <= 0 {
                        // If slots run out, break out resulting in added == []
                        break;
                    } else if guild_id == *key {
                        // If we get to the current guild, get a sorted list of added bots capped
                        // at the available slot count or 20, whichever is lower
                        added = bots
                            .iter()
                            .take(bot_count as usize)
                            .map(|x| x.to_string())
                            .collect();
                        subscription -= bot_count;
                        // If the list is longer than the free tier threshold, we're in the free tier
                        in_free_tier = bots.len() >= FREE_THRESHOLD as usize;
                        // Stop the search
                        break;
                    } else {
                        // Subtract used slots from total slots
                        subscription -= bot_count;
                    }
                }
            }

            if slots.is_none()
                || slots.unwrap().get(&guild_id).is_none()
                || !slots
                    .unwrap()
                    .get(&guild_id)
                    .unwrap()
                    .added
                    .as_ref()
                    .unwrap()
                    .contains(&bot_id.0.get().to_string())
            {
                // We still add the bot id to the server list of bots if it's not there
                let owner = properties.settings.setup.connection.unwrap();
                println!("[{}]: Adding bot to {} by {}", bot_id, guild_id, owner);
                let result = database
                    .fluent()
                    .update()
                    .in_col("accounts")
                    .document_id(&owner)
                    .transforms(|t| {
                        let field = format!("customer.slots.satellites.`{}`.added", guild_id);
                        t.fields([t
                            .field(field)
                            .append_missing_elements([bot_id.0.get().to_string()])])
                    })
                    .only_transform()
                    .add_to_transaction(&mut transaction);

                if let Err(err) = result {
                    eprintln!(
                        "[{}]: Couldn't add bot to {} by {}: {:?}",
                        bot_id, guild_id, owner, err
                    );
                    continue;
                }
				needs_commit = true;

                if in_free_tier || subscription > 0 {
                    added.push(bot_id.0.get().to_string());
                }
            } else if in_free_tier {
                // If we're in the free tier, add the bot to the local list of all bots in the server
                added.push(bot_id.0.get().to_string());
            }

            if added.contains(&bot_id.0.get().to_string()) {
                // If the bot is in the list of all bots in the server, update the nickname
                update_nickname(&ctx, bot_id, guild, &price_text).await;
            } else {
                update_nickname(&ctx, bot_id, guild, "More subscription slots required").await;
            }
        }
    }

    // Update global presence
    ctx.set_presence(Some(ActivityData::custom(state)), status);

	// Commit database transaction if necessary
	if needs_commit {
		// Commit database transaction
		let result = transaction.commit().await;
		if let Err(err) = result {
			eprintln!("[{}]: Couldn't commit transaction: {:?}", bot_id, err);
		}
	}

    let duration = start.elapsed();
    println!("[{}]: Updated nicknames in {:?}", bot_id, duration);
    return duration;
}

async fn update_nickname(ctx: &Arc<Context>, bot_id: UserId, guild: &GuildId, nickname: &str) {
    let current_nickname = match guild.to_guild_cached(&ctx.cache) {
        Some(guild) => guild.members.get(&bot_id).unwrap().nick.clone(),
        None => None,
    };
    if current_nickname == Some(nickname.to_string()) {
        return;
    }

    let result = guild.edit_nickname(ctx.http.as_ref(), Some(nickname)).await;
    if let Err(err) = result {
        println!(
            "[{}]: Couldn't update nickname in {}: {:?}",
            bot_id, guild, err
        );
    }
}

#[tokio::main]
async fn main() {
    sleep(Duration::from_secs(5)).await;

    let mut satellite_id: usize = match env::var("HOSTNAME") {
        Ok(hostname) => hostname.split("-").last().unwrap().parse().unwrap_or(0),
        Err(_) => 0,
    };

    if env::var("IS_FREE").is_err() {
        satellite_id += 2;
    }

    let token = env::var(format!("ID_{}", SATELLITES[satellite_id]))
        .expect("Expected a bot token in the environment");

    let intents = GatewayIntents::GUILDS;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<RequestCache>(Arc::new(RwLock::new(None)));
        data.insert::<UserInfo>(Arc::new(RwLock::new(None)));
    }

    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }
}
