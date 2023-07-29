mod command;

use frankenstein::AllowedUpdate;
use frankenstein::Api;
use frankenstein::GetChatMemberCountParams;
use frankenstein::GetUpdatesParams;
use frankenstein::Message;
use frankenstein::MessageEntityType;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;
use frankenstein::UpdateContent;

use log;

use command::*;

static TOKEN: &str = "<token>";
static BOT_NAME: &str = "poll_count_bot"; // TODO: get from telegram

fn main() {
    simple_logger::init_with_env().unwrap();

    log::info!("Starting bot...");
    let api: Api = Api::new(TOKEN);

    listen_to_message_updates(&api);
}

fn listen_to_message_updates(api: &Api) {
    let update_params_builder =
        GetUpdatesParams::builder().allowed_updates(vec![AllowedUpdate::Message]);

    let mut update_params = update_params_builder.clone().build();

    log::info!("Listening to updates...");
    loop {
        let result = api.get_updates(&update_params);

        match result {
            Ok(updates) => {
                for update in updates.result {
                    log::info!("Received update: {:?}", update.update_id);

                    if let UpdateContent::Message(msg) = update.content {
                        log::info!("Received message: {:?}", msg.message_id);

                        process_message(&api, &msg);
                    } else {
                        log::debug!("Received non-message update {:?}", update.content);
                    }

                    update_params = update_params_builder
                        .clone()
                        .offset(update.update_id + 1)
                        .build();
                }
            }
            Err(e) => log::error!("Error while getting updates: {:?}", e),
        }
    }
}

fn process_message(api: &Api, msg: &Message) {
    match (&msg.entities, &msg.text) {
        (Some(entities), Some(text)) => entities
            .iter()
            .find(|entity| entity.type_field == MessageEntityType::BotCommand)
            .map_or_else(
                || log::info!("Ignoring message without bot command: {:?}", msg.message_id),
                |entity| {
                    let command_str =
                        &text[entity.offset as usize..(entity.offset + entity.length) as usize];

                    match resolve_command(command_str, BOT_NAME) {
                        CommandResolution::ForMe(cmd) => match cmd {
                            Command::Defined(cmd) => handle_command(api, msg, cmd),
                            Command::Unknown => {
                                log::info!("Unknown command: {:?}", command_str);
                                reply(api, msg, "Unknown command");
                            }
                        },
                        CommandResolution::NotForMe => {
                            log::info!("Ignoring command for other bot: {:?}", command_str);
                        }
                        CommandResolution::Error => {
                            log::error!("Error while resolving command: {:?}", command_str);
                        }
                    }
                },
            ),
        _ => log::info!(
            "Ignoring message without entities or text: {:?}",
            msg.message_id
        ),
    }
}

fn handle_command(api: &Api, msg: &Message, cmd: BotCommand) {
    log::info!("Handling command: {:?}", cmd);

    let maybe_poll = msg
        .reply_to_message
        .as_ref()
        .and_then(|reply_to| match &reply_to.poll {
            Some(poll) => Some(poll),
            None => None,
        });

    match (&maybe_poll, &cmd) {
        (None, _) => reply(api, msg, "Please reply to a message with a poll"),
        (_, BotCommand::Ping) => reply(api, msg, "Not implemented yet"),
        (Some(poll), BotCommand::Count) => {
            log::info!("Found poll: {:?}", poll.id);

            let count = get_chat_members_count(api, msg);

            match count {
                None => reply(api, msg, "Error while getting chat members count"),
                Some(count) => {
                    log::info!("Found chat members count: {:?}", count);

                    let text = format!("{} out of {}", poll.total_voter_count, count);

                    reply(api, msg, &text);
                }
            }
        }
    }
}

fn get_chat_members_count(api: &Api, msg: &Message) -> Option<u32> {
    let get_chat_member_count_params = GetChatMemberCountParams::builder()
        .chat_id(msg.chat.id)
        .build();

    let result = api.get_chat_member_count(&get_chat_member_count_params);

    result.ok().map(|resp| resp.result)
}

fn reply(api: &Api, msg: &Message, text: &str) {
    let send_message_params = SendMessageParams::builder()
        .chat_id(msg.chat.id)
        .text(text)
        .reply_to_message_id(msg.message_id)
        .build();

    let result = api.send_message(&send_message_params);

    match result {
        Ok(_) => log::debug!("Sent reply to message {:?}", msg.message_id),
        Err(e) => log::error!("Error while sending reply: {:?}", e),
    }
}
