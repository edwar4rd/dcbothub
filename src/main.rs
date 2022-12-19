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

        let command_output = match &parsed {
            Ok(cli) => match cli.command {
                parser::Commands::Exit => {
                    break;
                }
                parser::Commands::List => {
                    let mut output = String::new();
                    for name in bots.keys() {
                        output.push_str(&name);
                        output.push(' ');
                    }
                    let mut output = output.trim_end().to_string();
                    output.push('\n');
                    output
                }
                parser::Commands::ListStatus => {
                    let mut output = String::new();
                    for (name, instance) in &mut bot_instances {
                        output.push_str(&format!(
                            "{} {} {}",
                            name,
                            if instance.is_ok() {
                                "started"
                            } else {
                                "failed"
                            },
                            instance.as_mut().map_or_else(
                                |error| error.to_string(),
                                |child| child.try_wait().unwrap().map_or_else(
                                    || "running".to_string(),
                                    |status| format!("exited {}", status.code().unwrap_or(-1))
                                )
                            )
                        ));
                        output.push('\n');
                    }
                    output
                }
                _ => todo!(),
            },
            Err(_) => "\n".to_string(),
        };

        if control_bot.is_some() {
            let bot_out = bot_out.as_mut().unwrap();
            write!(bot_out, "{}\n", command_output.lines().count())
                .expect("Failed outputing to control_bot");
            write!(bot_out, "{}", command_output).expect("Failed outputing to control_bot");
            bot_out.flush().expect("Failed outputing to control_bot");
        } else {
            print!("{}", command_output);
        }

        if parsed.is_err() {
            print!("{}", parsed.unwrap_err());
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
}
