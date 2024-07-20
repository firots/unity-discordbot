use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Utc, DateTime};
use anyhow::anyhow;
use poise::serenity_prelude::{ButtonStyle, ChannelId, CreateActionRow, CreateAttachment, CreateButton, CreateMessage, Http, ReactionType};
use rand::Rng;
use serde_json::Value;
use crate::bot::Bot;
use crate::gift_code::{add_days_to_current_date, generate_gift_code, get_gift_code_embed, is_valid_gift_code, validate_gift_code};
use crate::models::{GamePlatform, GameVersion, GiftCode, GiftCodeResponse};
use crate::{Context, Error};


impl Bot {
    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn addgiftcode(
        ctx: Context<'_>,
        title: String,
        subtitle: String,
        amount: u32,
        duration: u32,
        rewards: Value,
        test: bool,
        hidden: bool,
    ) -> Result<(), Error> {
        validate_gift_code(&title, &subtitle, amount, duration, &rewards)?;
        let unity_service = ctx.data().unity_service.clone();

        if ctx.channel_id() != ctx.data().bot.gift_code_test_channel_id {
            return Err(anyhow!("This command can only be used in the test channel").into());
        }

        let gift_code_count = unity_service.get_gift_code_count().await?;
        if gift_code_count > 19 {
            return Err(anyhow!(format!("Gift code limit reached. Gift code count: {}", gift_code_count)).into());
        }

        let channel_id: u64;
        if test || hidden {
            channel_id = ctx.data().bot.gift_code_test_channel_id;
        } else {
            channel_id = ctx.data().bot.gift_code_channel_id;
        }

        let code = generate_gift_code();
        let expiration_date = add_days_to_current_date(duration as i64);

        let message_id = Bot::generate_custom_id();
        let button_id = Bot::generate_custom_id();

        let gift_code = GiftCode {
            title: title.clone(),
            subtitle: subtitle.clone(),
            amount,
            duration,
            expired_at: expiration_date.clone(),
            rewards: serde_json::from_value(rewards.clone())?,
            channel_id,
            message_id: message_id.clone(),
            button_id: button_id.clone(),
        };

        if !test {
            unity_service.save_gift_code(&code, &gift_code).await?;
        }
        
        let channel_id = ChannelId::new(channel_id);
        let http = Http::new(&ctx.data().bot.discord_token);
        
        let builder = CreateMessage::default()
            .embed(get_gift_code_embed(&gift_code))
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new(button_id)
                    .style(ButtonStyle::Primary)
                    .label("Get Code")
                    .emoji(ReactionType::Unicode("🎁".to_string()))
            ])]);

        channel_id.send_message(&http, builder).await?;
        
        if test {
            let response = format!("Test gift code: Title: {}, Code: {}, ExpiredAt: {} Amount: {}, Rewards: {}", title, code, expiration_date, amount, rewards);
            ctx.say(response).await?;
        } else {
            {
                let gift_code_response = GiftCodeResponse {
                    key: code.clone(),
                    value: gift_code.clone(),
                };
                ctx.data().bot.insert_gift_code(gift_code_response).await;
            }
            let response = format!("Gift code added! Title: {}, Code: {}, ExpiredAt: {} Amount: {}, Rewards: {}", title, code, expiration_date, amount, rewards);
            ctx.say(response).await?;
        }

        Ok(())
    }

    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn removegiftcode(
        ctx: Context<'_>,
        code: String,
    ) -> Result<(), Error> {
        if code.is_empty() {
            return Err(anyhow!("Code cannot be empty").into());
        } else if is_valid_gift_code(&code) == false {
            return Err(anyhow!("Invalid gift code").into());
        } 

        let unity_service = ctx.data().unity_service.clone();
        unity_service.delete_gift_code(&code).await?;
        
        let response = format!("Gift code deleted! Code: {}", code);
        ctx.say(response).await?;
        Ok(())
    }

    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn updategameversion(ctx: Context<'_>, version_number: String, platform: String, force_update: bool) -> Result<(), Error> {
        let game_version = GameVersion {
            version_number: version_number.clone(),
            force_update,
        };

        let platform_object = GamePlatform::from_str(&platform)?;
        let unity_service = ctx.data().unity_service.clone();
        unity_service.update_game_version(&game_version, platform_object).await?;
        let response = format!("Game version updated successfully. Platform: {}, Version: {}, Forced: {}", platform.to_string(), version_number, force_update);
        ctx.say(response).await?;
        Ok(())
    } 

    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn updatesubscription(ctx: Context<'_>, player_id: String, product_id: String, duration: i32, increase_save_count_by: u64) -> Result<(), Error> {
        if ctx.channel_id() != ctx.data().bot.gift_code_test_channel_id {
            return Err(anyhow!("This command can only be used in the test channel").into());
        } else if ctx.data().bot.subscription_types.contains(&product_id) == false {
            return Err(anyhow!("Invalid product ID").into());
        } else if increase_save_count_by < 1 {
            return Err(anyhow!("Increase save count by must be greater than 0").into());
        }
        
        let unity_service = ctx.data().unity_service.clone();
        unity_service.update_subscription_data(&player_id, &product_id, duration, increase_save_count_by).await?;
        let response = format!("Subscription updated successfully. Player ID: {}, Product ID: {}, Duration: {}", player_id, product_id, duration);
        ctx.say(response).await?;
        Ok(())
    }

    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn getsavedata(ctx: Context<'_>, player_id: String) -> Result<(), Error> {
        if ctx.channel_id() != ctx.data().bot.gift_code_test_channel_id {
            return Err(anyhow!("This command can only be used in the test channel").into());
        }

        let unity_service = ctx.data().unity_service.clone();
        let save_data_json = unity_service.get_save_data(&player_id).await?;
        let save_data_string = serde_json::to_string_pretty(&save_data_json)?;
        let save_data_bytes = save_data_string.as_bytes();
        let filename = format!("save_data_{}.json", player_id);
        let attachment = CreateAttachment::bytes(save_data_bytes, filename);

        let http = Http::new(&ctx.data().bot.discord_token);
        let builder = CreateMessage::default()
            .add_file(attachment);
        ctx.channel_id().send_message(&http, builder).await?;
        ctx.say(format!("Save data sent for playerId: {}", player_id)).await?;
        Ok(())  
    }

    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn copysavedata(ctx: Context<'_>, to_player_id: String, from_player_id: String, increase_save_count_by: u64) -> Result<(), Error> {
        if ctx.channel_id() != ctx.data().bot.gift_code_test_channel_id {
            return Err(anyhow!("This command can only be used in the test channel").into());
        } else if increase_save_count_by < 1 {
            return Err(anyhow!("Increase save count by must be greater than 0").into());
        }

        let unity_service = ctx.data().unity_service.clone();
        let old_save_data = unity_service.get_save_data(&to_player_id).await?;
        let old_save_data_string = serde_json::to_string_pretty(&old_save_data)?;
        let old_save_data_bytes = old_save_data_string.as_bytes();
        let filename = format!("save_data_{}.json", to_player_id);
        let attachment = CreateAttachment::bytes(old_save_data_bytes, filename);

        let old_player_progress_data = old_save_data
            .get("playerProgressData")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("'playerProgressData' not found or null in {}", to_player_id))?;

        let old_save_count = old_player_progress_data
            .get("saveCount")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Unable to get or cast 'saveCount' as number in {}", to_player_id))?;

        
        let mut new_save_data = unity_service.get_save_data(&from_player_id).await?;

        let new_player_progress_data = new_save_data
            .get_mut("playerProgressData")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| anyhow!("'playerProgressData' not found or null in {}", from_player_id))?;

        new_player_progress_data.insert("saveCount".to_string(), Value::Number((old_save_count + increase_save_count_by).into()));

        unity_service.set_save_data(&to_player_id, new_save_data).await?;

        let http = Http::new(&ctx.data().bot.discord_token);
        let builder = CreateMessage::default()
            .add_file(attachment);
        ctx.channel_id().send_message(&http, builder).await?;

        ctx.say(format!("Save data copied to playerId: {} from playerId: {}. The old save is attached to this message.", to_player_id, from_player_id)).await?;
        Ok(())
    }

    #[poise::command(slash_command, prefix_command, owners_only)]
    pub async fn removestalegiftcodes(ctx: Context<'_>,) -> Result<(), Error> {
        let unity_service = ctx.data().unity_service.clone();
        let gift_codes = unity_service.get_all_gift_codes().await?;
        let now = Utc::now();
        let mut removed_codes = false;
        for gift_code in gift_codes.results {
            let expired_at_datetime = DateTime::parse_from_rfc3339(&gift_code.value.expired_at)
                .unwrap_or_else(|e| panic!("Failed to parse date: {}", e))
                .with_timezone(&Utc);

            if expired_at_datetime < now || gift_code.value.amount <= 0{
                unity_service.delete_gift_code(gift_code.key.as_str()).await?;
                ctx.say(format!("Gift code deleted! Code: {}", gift_code.key)).await?;
                removed_codes = true;
            }
        }
        if !removed_codes {
            ctx.say("No stale gift codes found").await?;
        }
        Ok(())
    }

    fn generate_custom_id() -> String {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let in_ms = since_the_epoch.as_millis();
    
        let mut rng = rand::thread_rng();
        let rand_num: u32 = rng.gen();
    
        format!("{}_{}", in_ms, rand_num)
    }
}


