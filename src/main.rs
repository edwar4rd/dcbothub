use clap::Parser;
use dcbothub::{parser, Bot};
use rustyline::error::ReadlineError;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufWriter, Write};
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

    let (mut bot_in, mut bot_out) = if let Some(control_bot) = &control_bot {
        match bot_instances.get_mut(control_bot).unwrap() {
            Ok(control_bot) => (
                Some(io::BufReader::new(control_bot.stdout.take().unwrap())),
                Some(BufWriter::new(control_bot.stdin.take().unwrap())),
            ),
            Err(err) => {
                println!("Failed starting control_bot:\n\t{}", err.to_string());
                return;
            }
        }
    } else {
        (None, None)
    };

    let mut rl = if control_bot.is_none() {
        let mut rl = rustyline::Editor::<()>::new().expect("Failed to create a terminal input");
        if rl.load_history("rustyline_history").is_err() {
            println!("No previous history.");
        }
        Some(rl)
    } else {
        None
    };

    // start listening to stdin/control_bot for commands
    loop {
        let mut input = String::new();
        if let Some(_) = &control_bot {
            let bot_in = bot_in.as_mut().unwrap();
            bot_in
                .read_line(&mut input)
                .expect("Failed reading line from control_bot");
        } else {
            let rl = rl
                .as_mut()
                .expect("Failed reading line from rustyline editor");
            match rl.readline(">> ") {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    input = line;
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    println!("Error reading line: {}", err.to_string());
                    break;
                }
            }
        };

        let parsed = parser::Cli::try_parse_from(
            "dcbothub"
                .split_whitespace()
                .chain(input.split_whitespace()),
        );

        // TODO: process the input as commands and react accordingly
        if control_bot.is_some() {
            bot_out
                .as_mut()
                .unwrap()
                .write_fmt(format_args!("Input: {input}"))
                .expect("Failed outputing to control_bot");
        } else {
            println!("Input: {input}");
            println!("Input: {:?}", parsed);
            match &parsed {
                Ok(cli) => match cli.command {
                    Some(parser::Commands::Exit) => {
                        break;
                    }
                    _ => todo!(),
                },
                Err(err) => match err.kind() {
                    clap::error::ErrorKind::DisplayHelp => {
                        println!("{}", err);
                    }
                    clap::error::ErrorKind::DisplayVersion => {
                        println!("{}", err);
                    }
                    clap::error::ErrorKind::InvalidSubcommand => {
                        println!("{}", err)
                    }
                    _ => todo!(),
                },
            }
        }
    }

    for (_, child) in bot_instances {
        match child {
            Ok(mut child) => {
                if child
                    .try_wait()
                    .expect("Failed to check child status")
                    .is_none()
                {
                    child.kill().expect("Failed to kill running child");
                }
            }
            Err(_) => (),
        }
    }

    if control_bot.is_none() {
        rl.unwrap().save_history("rustyline_history").unwrap();
    }
    /*
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
    }*/
}
