#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate rocket;

use async_channel::{Receiver, Sender};
use serenity::{client::ClientBuilder, prelude::*};

pub struct EventDispatcher(Sender<GithubEvent>);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum GithubEvent {
    ProposalOpened { issue_url: String },
}

async fn discord_bot(
    client: ClientBuilder,
    rx: Receiver<GithubEvent>,
) -> Result<core::convert::Infallible, serenity::Error> {
    let mut client = client.await?;
    client.start().await?;
    loop {
        let event = rx.recv().await.unwrap();
    }
}

#[rocket::get("/")]
fn test(dispatch: rocket::State<EventDispatcher>) -> String {
    format!("Test Returned Succesfully")
}

fn main() -> ! {
    let ghtoken = std::env::var("GITHUB_TOKEN")
        .expect("Missing or broken GITHUB_TOKEN (run . .ENV before running bot)");
    let distoken = std::env::var("DISCORD_TOKEN")
        .expect("Missing or broken DISCORD_TOKEN (run . .ENV before running bot)");

    let port = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap();

    let (tx, rx) = async_channel::bounded(256);

    let rt = tokio::runtime::Runtime::new().unwrap();

    let client = Client::builder(&distoken, GatewayIntents::all())
        .framework(serenity::framework::StandardFramework::new());

    let discord_fut = rt.spawn(discord_bot(client, rx));

    let cfg = rocket::Config::build(rocket::config::Environment::Development)
        .address("0.0.0.0")
        .port(port)
        .finalize()
        .unwrap();

    eprintln!(
        "Running Webhook server failed: {}",
        rocket::custom(cfg)
            .manage(EventDispatcher(tx))
            .mount("/", rocket::routes![test])
            .launch()
    );

    match rt.block_on(discord_fut).unwrap().expect("Discord Error") {}
}
