use serde_json::json;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::ChannelId},
    model::prelude::{UserId},
    http::Http,
    prelude::*,
};
use rand::seq::SliceRandom;
use std::{collections::HashSet, env, sync::Arc};
use tracing::{error, info};
use anyhow::{Result};


pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
struct General;

struct Handler;

async fn get_gif_link() -> Result<String> {
    let res = reqwest::
        get("https://api.giphy.com/v1/gifs/random?api_key=9eWjAOJrkSvRZYR9R7q426kKlWtzRon1&tag=cat")
        .await?.text().await?;

    let json : serde_json::Value = serde_json::from_str(&res).expect("JSON was not well-formatted");
    //println!("{}", serde_json::to_string_pretty(&json).unwrap());
    let result = json["data"]["images"]["original"]["url"].as_str().unwrap().to_owned();
    //println!("{}", result);
    Ok( result )
    /* 
    match url {
        Some(_) => println!("{:?}", url),
        None => println!("Error getting JSON Value"),
    }
    */
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, an image, three fields, and a footer.
            let msg = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.content("Hello, World!")
                        .embed(|e| {
                            e.title("This is a title")
                                .description("This is a description")
                                //.image("./yeet.gif")
                                .fields(vec![
                                    ("This is the first field", "This is a field body", true),
                                    ("This is the second field", "Both fields are inline", true),
                                ])
                                .field("This is the third field", "This is not an inline field", false)
                                .footer(|f| f.text("This is a footer"))
                                // Add a timestamp for the current time
                                // This also accepts a rfc3339 Timestamp
                                .timestamp(chrono::Utc::now())
                        })
                        .add_file("./yeet.gif")
                })
                .await;
                if let Err(why) = msg {
                    println!("Error sending message: {:?}", why);
                }
        } else if msg.content == "!ping" {
            let msg = msg
            .channel_id
            .send_message(&ctx.http, |m|{
                m.content("Pong!")
            })
            .await;
            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
            }


        } else if msg.content == "!cat" {
            let content_maybe = get_gif_link().await;
            let mut content = String::new();
            match content_maybe{
                Ok(url) => content = url,
                Err(e) => println!("Error Retrieving GIF URL: {}", e),
            }
            
            let msg = msg
            .channel_id
            .send_message(&ctx.http, |m|{
                m.content(content)
            })
            .await;
            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!help" {
            let msg = msg
            .channel_id
            .send_message(&ctx.http, |m|{
                m.content("
Work in Progress
Implemented Commands
    !help <- This message
    !cat <- Gives a random cat gif
    !ping <- Returns \"Pong!\"
    !hello <- Debug purposes
                ")
            }).await;
            if let Err(why) = msg {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework = StandardFramework::new().group(&GENERAL_GROUP)
        .configure(
            |c| c
            .owners(owners)
            .prefix("!")
            .with_whitespace(true)
            .allow_dm(false)
            .allowed_channels(vec![ChannelId(939346145581334588)].into_iter().collect())
            );

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
	
	
}