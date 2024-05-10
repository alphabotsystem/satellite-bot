mod config;
use chrono::{DateTime, Local, Utc};
use config::{CONFIGURATION, FREE_THRESHOLD, PROJECT, REQUEST_REFRESH_SECONDS, SATELLITES};
use database::{DatabaseConnector, GuildProperties};
use firestore::*;
use processor::{process_quote_arguments, process_task};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::{
    all::{Guild, OnlineStatus, ShardId, UnavailableGuild, UserId},
    async_trait,
    gateway::ActivityData,
    model::{gateway::Ready, id::GuildId},
    prelude::*,
    utils::shard_id,
};
use std::{
    collections::HashSet,
    env,
    panic::{set_hook, take_hook},
    process::exit,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::broadcast,
    sync::{Mutex, RwLock},
    time::sleep,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UserInfo {
    pub icon: String,
    pub name: String,
    pub status: String,
    pub price: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SatelliteProperties {
    pub count: usize,
    pub servers: Vec<String>,
    pub user: UserInfo,
}

struct Cache {
    pub request: RwLock<Option<Value>>,
    pub user_info: RwLock<Option<UserInfo>>,
}

struct Handler {
    tasks: Arc<Mutex<HashSet<ShardId>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let platform = CONFIGURATION
            .get(&ctx.cache.current_user().id.to_string())
            .unwrap()
            .0;
        let refresh_rate = match platform {
            "Alternative.me" => Duration::from_secs(60 * 5),
            "CNN Business" => Duration::from_secs(60 * 5),
            "CoinGecko" => Duration::from_secs(60 * 5),
            "On-Chain" => Duration::from_secs(60 * 5),
            _ => Duration::from_secs(60),
        };

        if let Some(shard) = ready.shard {
            {
                let mut tasks = self.tasks.lock().await;
                if tasks.contains(&shard.id) {
                    return;
                }
                tasks.insert(shard.id);
            }

            ctx.set_presence(None, OnlineStatus::Idle);

            let ctx1 = ctx.clone();
            tokio::spawn(async move {
                loop {
                    update_ticker(&ctx1).await;
                    update_properties(&ctx1).await;
                    sleep(Duration::from_secs(REQUEST_REFRESH_SECONDS)).await;
                }
            });

            sleep(Duration::from_secs(5)).await;

            let ctx2 = ctx.clone();
            tokio::spawn(async move {
                loop {
                    let duration = update_nicknames(&ctx2).await;
                    if duration < refresh_rate {
                        sleep(refresh_rate - duration).await;
                    }
                }
            });

            println!(
                "[Startup]: Alpha.bot Satellite ({}) is online (shard {}/{})",
                ready.user.id,
                shard.id.0 + 1,
                shard.total,
            );
        }
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        if !_is_new.unwrap_or(false) {
            return;
        }

        let bot_id = _ctx.cache.current_user().id;
        let guild_id = guild.id.to_string();

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

        let database = match FirestoreDb::new(PROJECT).await {
            Ok(database) => database,
            Err(err) => {
                eprintln!("[{}]: Couldn't connect to Firestore: {:?}", bot_id, err);
                return;
            }
        };
        let mut transaction = match database.begin_transaction().await {
            Ok(transaction) => transaction,
            Err(err) => {
                eprintln!(
                    "[{}]: Couldn't start Firestore transaction: {:?}",
                    bot_id, err
                );
                return;
            }
        };

        let result = database
            .fluent()
            .update()
            .in_col("accounts")
            .document_id(properties.settings.setup.connection.unwrap())
            .transforms(|t| {
                let field = format!("customer.slots.satellites.`{}`.added", guild_id);
                t.fields([t.field(field).append_missing_elements([bot_id.to_string()])])
            })
            .only_transform()
            .add_to_transaction(&mut transaction);

        match result {
            Ok(_) => (),
            Err(err) => {
                eprintln!("[{}]: Couldn't add bot to {}: {:?}", bot_id, guild_id, err);
                return;
            }
        }

        match transaction.commit().await {
            Ok(_) => (),
            Err(err) => {
                eprintln!("[{}]: Couldn't commit transaction: {:?}", bot_id, err);
            }
        }
    }

    async fn guild_delete(&self, _ctx: Context, guild: UnavailableGuild, _full: Option<Guild>) {
        let bot_id = _ctx.cache.current_user().id;
        let guild_id = guild.id.to_string();

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

        let database = match FirestoreDb::new(PROJECT).await {
            Ok(database) => database,
            Err(err) => {
                eprintln!("[{}]: Couldn't connect to Firestore: {:?}", bot_id, err);
                return;
            }
        };

        let mut transaction = match database.begin_transaction().await {
            Ok(transaction) => transaction,
            Err(err) => {
                eprintln!(
                    "[{}]: Couldn't start Firestore transaction: {:?}",
                    bot_id, err
                );
                return;
            }
        };

        let result = database
            .fluent()
            .update()
            .in_col("accounts")
            .document_id(properties.settings.setup.connection.unwrap())
            .transforms(|t| {
                let field = format!("customer.slots.satellites.`{}`.added", guild_id);
                t.fields([t.field(field).remove_all_from_array([bot_id.to_string()])])
            })
            .only_transform()
            .add_to_transaction(&mut transaction);

        match result {
            Ok(_) => (),
            Err(err) => {
                eprintln!(
                    "[{}]: Couldn't remove bot from {}: {:?}",
                    bot_id, guild_id, err
                );
                return;
            }
        }

        match transaction.commit().await {
            Ok(_) => (),
            Err(err) => {
                eprintln!("[{}]: Couldn't commit transaction: {:?}", bot_id, err);
            }
        }
    }
}

async fn update_ticker(ctx: &Context) {
    let bot_id = ctx.cache.current_user().id;
    let (platform, arguments, ticker_id) = CONFIGURATION.get(&bot_id.to_string()).unwrap();

    let arguments = match arguments {
        Some(arguments) => arguments.split(" ").collect(),
        None => vec![],
    };

    println!(
        "[{}]: Updating cached request for {}:{} + {:?}",
        bot_id, platform, ticker_id, arguments
    );

    // Make parser request
    let (message, request) =
        process_quote_arguments(arguments, vec![platform], Some(ticker_id)).await;

    if request.is_null() || !message.is_null() {
        eprintln!("[{}]: Parsing failed: {}", bot_id, message);
        eprintln!("[{}]: {:?}", bot_id, request);
        return;
    }

    // Update global cache
    {
        let data = ctx.data::<Cache>();
        data.request.write().await.replace(request);
    }
}

async fn update_properties(ctx: &Context) {
    let bot_id = ctx.cache.current_user().id;

    // Initialize database connection
    let database = match FirestoreDb::new(PROJECT).await {
        Ok(database) => database,
        Err(err) => {
            eprintln!("[{}]: Couldn't connect to Firestore: {:?}", bot_id, err);
            return;
        }
    };

    // Obtain cached user info object
    let user_info = {
        let data = ctx.data::<Cache>();

        let user_info = data.user_info.read().await.clone();
        match user_info {
            Some(user_info) => user_info,
            None => {
                println!("[{}]: User info has not been cached yet", bot_id);
                return;
            }
        }
    };

    // Update properties
    let guilds = ctx.cache.guilds();
    let properties = SatelliteProperties {
        count: guilds.len(),
        servers: guilds.iter().map(|x| x.to_string()).collect(),
        user: user_info,
    };

    // Push properties to database
    let result = database
        .fluent()
        .update()
        .in_col("satellites")
        .document_id(bot_id.to_string())
        .object(&properties)
        .execute::<()>()
        .await;

    if let Err(err) = result {
        eprintln!("[{}]: Couldn't update properties: {:?}", bot_id, err);
    }
}

async fn update_nicknames(ctx: &Context) -> Duration {
    let start = Instant::now();
    let bot_id = ctx.cache.current_user().id;
    let shard_count = ctx.cache.as_ref().shard_count();
    let is_free = bot_id.get() == 751080162300526653 || bot_id.get() == 751080770243657779;

    println!(
        "[{}]: Updating nicknames in shard {}/{}",
        bot_id,
        ctx.shard_id.0 + 1,
        shard_count
    );

    // Obtain cached request object
    let request = {
        let data = ctx.data::<Cache>();

        let request = data.request.read().await.clone();
        match request {
            Some(request) => request,
            None => {
                println!("[{}]: Request has not been cached yet", bot_id);
                return Duration::from_secs(0);
            }
        }
    };

    // Make quote request
    let response = process_task(request.clone(), "quote", None, None, None, None).await;
    let data = match response {
        Ok(response) => response,
        Err(err) => {
            eprintln!(
                "[{}]: Something went wrong when making a network request the price: {}",
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
        "Blockchair" => (
            {
                let quote = payload
                    .get("quotePrice")
                    .expect("Expected quotePrice in payload")
                    .as_str()
                    .unwrap();

                let timestamp = quote
                    .chars()
                    .skip(5)
                    .take(10)
                    .collect::<String>()
                    .parse::<i64>()
                    .expect(format!("Couldn't parse timestamp from {}", quote).as_str());

                let now = Local::now();
                let halving = DateTime::<Utc>::from_timestamp(timestamp, 0)
                    .expect("Couldn't parse date from timestamp");
                let duration = halving.signed_duration_since(now);

                let years = duration.num_days() / 365;
                let days = duration.num_days() % 365;
                let hours = duration.num_hours() % 24;
                let minutes = duration.num_minutes() % 60;

                if years > 0 {
                    format!(
                        "{} year{} {} day{}",
                        years,
                        if years == 0 { "" } else { "s" },
                        days,
                        if days == 1 { "" } else { "s" },
                    )
                } else if days > 0 {
                    format!(
                        "{} day{} {} hour{}",
                        days,
                        if days == 1 { "" } else { "s" },
                        hours,
                        if hours == 1 { "" } else { "s" },
                    )
                } else if hours > 0 {
                    format!(
                        "{} hour{} {} minute{}",
                        hours,
                        if hours == 1 { "" } else { "s" },
                        minutes,
                        if minutes == 1 { "" } else { "s" },
                    )
                } else {
                    format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" },)
                }
            },
            {
                let quote = payload
                    .get("quotePrice")
                    .expect("Expected quotePrice in payload")
                    .as_str()
                    .unwrap();

                let timestamp = quote
                    .chars()
                    .skip(5)
                    .take(10)
                    .collect::<String>()
                    .parse::<i64>()
                    .expect(format!("Couldn't parse timestamp from {}", quote).as_str());

                let halving = DateTime::<Utc>::from_timestamp(timestamp, 0)
                    .expect("Couldn't parse date from timestamp");
                format!("{} UTC | ", halving.format("%B %d %Y at %H:%M"))
            },
            format!(
                "{} | ",
                ticker
                    .get("id")
                    .expect("Expected id in ticker")
                    .as_str()
                    .unwrap()
                    .split(":")
                    .last()
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
    {
        let data = ctx.data::<Cache>();

        data.user_info.write().await.replace(UserInfo {
            icon: ctx
                .cache
                .current_user()
                .avatar_url()
                .unwrap_or("".to_string()),
            name: ctx.cache.current_user().name.clone().into_string(),
            status: state.clone(),
            price: price_text.clone(),
        });
    }

    // Initialize database connections
    let guild_properties = DatabaseConnector::<GuildProperties>::new();
    let database = match FirestoreDb::new(PROJECT).await {
        Ok(database) => database,
        Err(err) => {
            eprintln!("[{}]: Couldn't connect to Firestore: {:?}", bot_id, err);
            return start.elapsed();
        }
    };

    let mut transaction = match database.begin_transaction().await {
        Ok(transaction) => transaction,
        Err(err) => {
            eprintln!(
                "[{}]: Couldn't start Firestore transaction: {:?}",
                bot_id, err
            );
            return start.elapsed();
        }
    };

    let mut needs_commit = false;

    // Update guild nicknames
    let guilds = ctx.cache.guilds();
    for guild in guilds.iter() {
        if shard_id(*guild, shard_count) != ctx.shard_id.0 {
            continue;
        }

        if is_free {
            // If the bot is in the free tier, update the nickname immediately
            update_nickname(&ctx, bot_id, guild, &price_text).await;
        } else {
            // Get guild properties
            let guild_id = guild.to_string();
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
                    .contains(&bot_id.to_string())
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
                        t.fields([t.field(field).append_missing_elements([bot_id.to_string()])])
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
                    added.push(bot_id.to_string());
                }
            } else if in_free_tier {
                // If we're in the free tier, add the bot to the local list of all bots in the server
                added.push(bot_id.to_string());
            }

            if added.contains(&bot_id.to_string()) {
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

async fn update_nickname(ctx: &Context, bot_id: UserId, guild: &GuildId, nickname: &str) {
    let current_nickname = match guild.to_guild_cached(&ctx.cache) {
        Some(guild) => guild.members.get(&bot_id).unwrap().nick.clone(),
        None => None,
    };
    if current_nickname.is_some() && current_nickname.unwrap().into_string() == nickname.to_string()
    {
        return;
    }

    let result = guild
        .edit_nickname(
            ctx.http.as_ref(),
            Some(nickname),
            Some("Automatic update to reflect current information"),
        )
        .await;
    if let Err(err) = result {
        match err {
            SerenityError::Http(err) => {
				match err {
					HttpError::UnsuccessfulRequest(response) => {
						if response.error.message == "Missing Permissions" {
							println!("[{}]: Missing permissions in {}", bot_id, guild);
							// let result = guild.leave(ctx.http.as_ref()).await;
							// if let Err(err) = result {
							//     eprintln!("[{}]: Couldn't leave {}: {:?}", bot_id, guild, err);
							// }
						} else if !response.status_code.is_server_error() {
							eprintln!(
								"[{}]: Request to update nickname in {} failed: {:?}",
								bot_id, guild, response
							);
						}
					},
					HttpError::Request(err) => {
						if !err.is_connect() {
							println!(
								"[{}]: Couldn't make HTTP request to update nickname in {}: {:?}",
								bot_id, guild, err
							)
						} else {
							eprintln!(
								"[{}]: Couldn't make HTTP request to update nickname in {}: {:?}",
								bot_id, guild, err
							);
						}
					},
					_ => {
						eprintln!(
							"[{}]: Request to update nickname in {} failed: {:?}",
							bot_id, guild, err
						);
					}
				}
            },
            _ => {
                eprintln!(
                    "[{}]: Couldn't update nickname in {}: {:?}",
                    bot_id, guild, err
                );
            }
        }
    }
}

async fn run_bot_with(id: &str, signal: broadcast::Sender<()>) {
    let data = Cache {
        request: RwLock::new(None),
        user_info: RwLock::new(None),
    };

    let token = env::var(format!("ID_{}", id)).expect("Expected a bot token in the environment");

    let intents = GatewayIntents::GUILDS;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            tasks: Arc::new(Mutex::new(HashSet::new())),
        })
        .data(Arc::new(data) as _)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start_autosharded().await {
        eprintln!("Client error: {:?}", why);
    }

    signal.send(()).expect("Couldn't send signal");
}

#[tokio::main]
async fn main() {
    let default_panic = take_hook();
    set_hook(Box::new(move |info| {
        default_panic(info);
        exit(1);
    }));

    let (tx, mut rx) = broadcast::channel(1);

    let mut threads = Vec::new();
    for id in SATELLITES {
        sleep(Duration::from_secs(1)).await;

        let tx = tx.clone();
        threads.push(tokio::spawn(async move {
            run_bot_with(id, tx).await;
        }));
    }

    rx.recv().await.expect("Couldn't receive exit signal");
    eprintln!("Something really bad happened");

    sleep(Duration::from_secs(60)).await;
}
