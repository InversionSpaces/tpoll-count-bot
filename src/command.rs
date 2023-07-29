use regex::Regex;

#[derive(Debug)]
pub enum BotCommand {
    Count,
    Ping,
}

#[derive(Debug)]
pub enum Command {
    Defined(BotCommand),
    Unknown,
}

#[derive(Debug)]
pub enum CommandResolution {
    ForMe(Command),
    NotForMe,
    Error,
}

pub fn resolve_command(cmd: &str, bot_name: &str) -> CommandResolution {
    let re = Regex::new(r"^/([^@]*)@?(.+)?").unwrap();
    let caps = re.captures(cmd).unwrap();

    let match_command = |cmd: &str, default: CommandResolution| match cmd {
        "count" => CommandResolution::ForMe(Command::Defined(BotCommand::Count)),
        "ping" => CommandResolution::ForMe(Command::Defined(BotCommand::Ping)),
        _ => default,
    };

    match &caps.get(1) {
        None => CommandResolution::Error,
        Some(cap1) => match &caps.get(2) {
            None => match_command(cap1.as_str(), CommandResolution::NotForMe),
            Some(cap2) if cap2.as_str() == bot_name => {
                match_command(cap1.as_str(), CommandResolution::ForMe(Command::Unknown))
            }
            _ => CommandResolution::NotForMe,
        },
    }
}
