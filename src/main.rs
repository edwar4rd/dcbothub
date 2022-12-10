use dcbothub::Bot;
use std::collections::HashMap;
use std::fs;
use toml;

fn parse_bots() -> Result<(HashMap<String, Bot>, Option<String>), ()> {
    let file = match fs::read_to_string("bots.toml") {
        Ok(file) => file,
        Err(_) => {
            println!("Failed to open bots.toml, check your working directory...");
            return Err(());
        }
    };

    let toml = match file.parse::<toml::Value>() {
        Ok(toml) => toml,
        Err(err) => {
            println!(
                "Failed to parse bots.toml as a valid toml file:\n\t{}",
                err.to_string()
            );
            return Err(());
        }
    };

    let bots = match toml.get("bot") {
        Some(toml::Value::Array(bots)) => bots,
        Some(_) => {
            println!("bot in bots.toml should be a array of table!");
            return Err(());
        }
        None => {
            println!("No bot is presented in bots.toml!");
            return Err(());
        }
    };

    let mut hashmap = HashMap::new();

    for bot in bots {
        let bot = match Bot::from_toml_table(match bot.as_table() {
            Some(bot) => bot,
            None => {
                println!("bot in bots.toml should be a array of table!");
                return Err(());
            }
        }) {
            Ok(bot) => bot,
            Err(err) => {
                println!("Failed to parse bot in bots.toml:\n\t{}", err);
                return Err(());
            }
        };
        if hashmap.insert(bot.name().to_string(), bot).is_some() {
            println!("Multiple bots in bots.toml have identical name!");
            return Err(());
        };
    }

    let control_bot = match toml.get("control_bot") {
        Some(toml::Value::String(control_bot)) => {
            if !hashmap.contains_key(control_bot) {
                println!("control_bot should contain a bot name presented in a bot table");
                return Err(());
            }
            Some(control_bot.to_string())
        }
        Some(_) => {
            println!("control_bot should contain a bot name presented in a bot table");
            return Err(());
        }
        None => None,
    };

    Ok((hashmap, control_bot))
}

fn main() {
    // read in and verify bots.toml
    let (bots, control_bot) = match parse_bots() {
        Ok(botnctrl) => botnctrl,
        Err(_) => {
            return;
        }
    };

    // start every bot in bots.toml
    let mut bot_instances = HashMap::new();
    for (name, bot) in &bots {
        bot_instances.insert(
            name,
            bot.run()
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn(),
        );
    }

    // start listening to stdin/control_bot for commands
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        for (name, instance) in &mut bot_instances {
            println!("Name: {}", name);
            println!("Status: {}", match instance.as_mut().unwrap().try_wait().unwrap() {
                Some(status) => status.to_string(),
                None => "Running...".to_string()
            });
        }
        println!("");
        if control_bot.is_some() {
            let instance = bot_instances.get_mut(control_bot.as_ref().unwrap()).unwrap();
            println!("control_bot Name: {}", control_bot.as_ref().unwrap());
            println!("Status: {}", match instance.as_mut().unwrap().try_wait().unwrap() {
                Some(status) => status.to_string(),
                None => "Running...".to_string()
            });
            println!("\n");
        }
    }
}
