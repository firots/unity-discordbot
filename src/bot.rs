use std::collections::{HashMap, HashSet};
use std::env;
use tokio::sync::RwLock;
use std::sync::Arc;
use poise::serenity_prelude::{ChannelId, ClientBuilder, ComponentInteraction, ComponentInteractionCollector, Context as SerenityContext, CreateInteractionResponseFollowup, EditMessage, GatewayIntents, MessageFlags};
use crate::config::{read_owners, read_subscription_types};
use crate::constans::{DISCORD_TOKEN, GIFT_CODE_CHANNEL, GIFT_CODE_TEST_CHANNEL, INTERACTION_LISTENER_RETRY_DELAY};
use crate::db::Db;
use crate::gift_code::get_gift_code_embed;
use crate::models::GiftCodeResponse;
use crate::unity_service::UnityService;
use crate::{ContextData, Error};
use chrono::{DateTime, Utc};

pub struct Bot {
    db: Db,
    gift_codes: RwLock<HashMap<String, GiftCodeResponse>>,
    pub discord_token: String,
    pub gift_code_channel_id: u64,
    pub gift_code_test_channel_id: u64,
    unity_service: Arc<UnityService>,
    pub subscription_types: HashSet<String>,
}

impl Bot {
    pub fn new(db: Db) -> Result<Self, Error> {
        Ok(Self {
            db,
            gift_codes:RwLock::new(HashMap::new()),
            discord_token: env::var(DISCORD_TOKEN)?,
            gift_code_channel_id: env::var(GIFT_CODE_CHANNEL)?.parse::<u64>()?,
            gift_code_test_channel_id: env::var(GIFT_CODE_TEST_CHANNEL)?.parse::<u64>()?,
            unity_service: Arc::new(UnityService::new()?),
            subscription_types: read_subscription_types(),
        })
    }
}

impl Bot {
    pub async fn load_gift_codes(&self) -> Result<(), Error> {
        let server_gift_codes = self.unity_service.get_all_gift_codes().await?;
        for gift_code in server_gift_codes.results {
            self.insert_gift_code(gift_code).await;
        }
        Ok(())
    }

    pub async fn insert_gift_code(&self, gift_code: GiftCodeResponse) {
        let mut gift_codes_write = self.gift_codes.write().await;
        gift_codes_write.insert(gift_code.value.button_id.clone(), gift_code.clone());
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Error> {
        let token = self.discord_token.clone();
        let owners = read_owners();
        let intents = GatewayIntents::non_privileged();
        let unity_service = self.unity_service.clone();

        self.load_gift_codes().await?;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                owners,
                commands: vec![
                    Bot::addgiftcode(),
                    Bot::removegiftcode(),
                    Bot::removestalegiftcodes(),
                    Bot::updategameversion(),
                    Bot::updatesubscription(),
                    Bot::getsavedata(),
                    Bot::copysavedata(),
                ],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                let self_clone = self.clone();
                Box::pin(async move {
                    tokio::spawn(self_clone.start_giftcode_button_listeners(ctx.clone()));
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(ContextData { 
                        bot: self,
                        unity_service,
                    })
                })
            })
            .build();
    
        let client = ClientBuilder::new(token, intents)
            .framework(framework)
            .await;
        client.unwrap().start().await.unwrap();

        Ok(())
    }

    async fn start_giftcode_button_listeners(self: Arc<Self>, ctx: SerenityContext) -> Result<(), Error> {
        let self_clone = self.clone();
        let gift_code_channel_id = self.gift_code_channel_id;
        let gift_code_test_channel_id = self.gift_code_test_channel_id;

        tokio::spawn(self_clone.listen_for_giftcode_button_clicks(ctx.clone(), gift_code_channel_id));
        tokio::spawn(self.listen_for_giftcode_button_clicks(ctx.clone(), gift_code_test_channel_id));
    
        Ok(())
    }

    async fn listen_for_giftcode_button_clicks(self: Arc<Self>, ctx: SerenityContext, channel_id: u64) {
        println!("Listening for gift code button clicks on channel: {}", channel_id);
        loop {
            match Self::wait_for_interaction(&ctx, channel_id).await {
                Some(mci) => {
                    let ctx = ctx.clone();
                    let gift_code_key: String;
                    {
                        let gift_codes = self.gift_codes.read().await;
                        match gift_codes.get(&mci.data.custom_id) {
                            Some(gift_code) => {
                                gift_code_key = gift_code.key.clone();
                            },
                            None => {
                                println!("Gift code not found for custom_id: {}", mci.data.custom_id);
                                continue;
                            }
                        }
                    }
                    let self_clone = self.clone();
                    tokio::spawn(async move {
                        match self_clone.handle_interaction(ctx, &gift_code_key, mci).await {
                            Ok(_) => (),
                            Err(e) => eprintln!("Error handling interaction. gift_code_key: {} error: {:?}", gift_code_key, e),
                        }
                    });
                },
                None => {
                    tokio::time::sleep(std::time::Duration::from_secs(INTERACTION_LISTENER_RETRY_DELAY)).await;
                    continue;
                }
            }
        }
    }

    async fn handle_interaction(self: Arc<Self>, ctx: SerenityContext, gift_code_key: &String, mci: ComponentInteraction) -> Result<(), Error> {
        mci.defer(ctx.clone()).await?;
    
        let mut message: String;
        let mut send_code = false;
        let mut gift_code = self.unity_service.get_gift_code(gift_code_key.clone()).await?;
    
        let expired_at_datetime = DateTime::parse_from_rfc3339(&gift_code.expired_at)?
            .with_timezone(&Utc);
    
        if gift_code.amount <= 0 {
            message = "Sorry, there are no more gift codes available.".to_string();
        } else if expired_at_datetime < Utc::now() {
            message = "Sorry, this gift code has expired.".to_string();
        } else {
            send_code = true;
            let user_id = mci.user.id.get();
            let is_already_redeemed = self.db.is_user_redeemed_gift_code_in_db(&gift_code_key, user_id).await?;
            if is_already_redeemed {
                message = "Sorry, you already redeemed this gift code. Your previous code was:".to_string();
            } else {
                let self_clone = self.clone();
                self_clone.decrease_gift_code_amount(&gift_code_key).await?;
                gift_code.amount -= 1;
                let mut msg = mci.message.clone();
                msg.edit(ctx.clone(), EditMessage::new().embed(get_gift_code_embed(&gift_code))).await?;
                self.db.redeem_gift_code_in_db( &gift_code_key, user_id).await?;
                message = "Congratulations! You have redeemed the gift code.".to_string();
            }
        }
    
        if send_code {
            message.push_str(format!("\n{}", gift_code_key).as_str());
        }
    
        let builder = CreateInteractionResponseFollowup::default()
            .content(message.clone())
            .flags(MessageFlags::EPHEMERAL);
    
        mci.create_followup(ctx.clone(), builder).await?;
            
        Ok(())
    }

    async fn decrease_gift_code_amount(self: Arc<Self>, gift_code_key: &String) -> Result<(), Error> {
        let mut gift_code = self.unity_service.get_gift_code(gift_code_key.clone()).await?;
        gift_code.amount -= 1;
        self.unity_service.save_gift_code(&gift_code_key, &gift_code).await?;
        Ok(())
    }

    async fn wait_for_interaction(ctx: &SerenityContext, channel_id: u64) -> Option<ComponentInteraction> {
        ComponentInteractionCollector::new(ctx)
            .channel_id(ChannelId::new(channel_id))
            .timeout(std::time::Duration::from_secs(INTERACTION_LISTENER_RETRY_DELAY))
            .await
    }
}