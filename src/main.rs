use std::sync::Arc;
use unity_discordbot::{config::load_config, db::Db};

#[tokio::main]
async fn main() {
    load_config();
    let db = Db::new().await;
    db.create_tables_if_needed().await.unwrap();

    match unity_discordbot::bot::Bot::new(db) {
        Ok(bot) => {
            let bot = Arc::new(bot);
            match bot.run().await {
                Ok(_) => println!("Bot is running"),
                Err(e) => eprintln!("Failed to run the bot: {:?}", e),
            }
        },
        Err(e) => eprintln!("Failed to create bot: {:?}", e),
    }
}