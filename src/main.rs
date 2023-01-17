use clap::Parser;
use dcbothub::{bot_parser, cmd_parser, BotInstances, Bots, TaskType, Tasks};
use rustyline::error::ReadlineError;
use std::collections::HashMap;
use std::io::{self, BufRead, BufWriter, Read, Write};

fn main() {
    // read in and verify bots.toml
    let (bots, control_bot) = match bot_parser::parse_bots() {
        Ok(botnctrl) => botnctrl,
        Err(_) => {
            return;
        }
    };

    // start every bot in bots.toml
    let mut bot_instances = HashMap::new();
    for (name, bot) in &bots {
        bot_instances.insert(
            name.clone(),
            bot.run()
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|err| err.to_string()),
        );
    }

    let mut tasks: Tasks = HashMap::new();

    match &control_bot {
        Some(control_bot) => {
            let _control_bot = match bot_instances.get_mut(control_bot).unwrap() {
                Ok(control_bot) => control_bot,
                Err(err) => {
                    println!("Failed starting control_bot:\n\t{}", err.to_string());
                    return;
                }
            };

            let bot_in =
                std::sync::Mutex::new(io::BufReader::new(_control_bot.stdout.take().unwrap()));
            let bot_out = std::sync::Mutex::new(BufWriter::new(_control_bot.stdin.take().unwrap()));

            cmd_loop(
                &bots,
                &mut bot_instances,
                &mut tasks,
                || {
                    let mut input = String::new();
                    bot_in
                        .lock()
                        .unwrap()
                        .read_line(&mut input)
                        .expect("Failed reading line from control_bot");
                    Ok(input)
                },
                |o| {
                    let mut bot_out = bot_out.lock().unwrap();
                    write!(bot_out, "{}\n", o.lines().count())
                        .expect("Failed writing output to control_bot");
                    write!(bot_out, "{}", o).expect("Failed writing output to control_bot");
                    bot_out
                        .flush()
                        .expect("Failed flushing output to control_bot");
                    Ok(())
                },
                |o| Ok(eprint!("{o}")),
                |bot_instances| {
                    let _control_bot = bot_instances.get_mut(control_bot).unwrap();
                    _control_bot
                        .as_mut()
                        .unwrap()
                        .kill()
                        .map_err(|err| err.to_string())?;
                    bot_instances.remove(control_bot);
                    let mut _control_bot = bots
                        .get(control_bot)
                        .unwrap()
                        .run()
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|err| err.to_string())?;
                    *bot_in.lock().unwrap() =
                        io::BufReader::new(_control_bot.stdout.take().unwrap());
                    *bot_out.lock().unwrap() = BufWriter::new(_control_bot.stdin.take().unwrap());
                    bot_instances.insert(control_bot.clone(), Ok(_control_bot));
                    Ok(())
                },
            )
            .unwrap();
        }
        None => {
            let mut rl = rustyline::Editor::<()>::new().expect("Failed to create a terminal input");
            if rl.load_history("rustyline_history").is_err() {
                println!("No previous history.");
            }

            cmd_loop(
                &bots,
                &mut bot_instances,
                &mut tasks,
                || loop {
                    match rl.readline(">>> ") {
                        Ok(line) => {
                            if line != "" {
                                rl.add_history_entry(line.as_str());
                                break Ok(line);
                            }
                        }
                        Err(ReadlineError::Interrupted) => {
                            println!("^C");
                            break Ok("exit".to_string());
                        }
                        Err(ReadlineError::Eof) => {
                            println!("^D");
                            break Ok("exit".to_string());
                        }
                        Err(err) => break Err(format!("Error reading line: {}", err.to_string())),
                    }
                },
                |o| Ok(print!("{o}")),
                |o| Ok(eprint!("{o}")),
                |_| Ok(()),
            )
            .unwrap();

            rl.save_history("rustyline_history").unwrap();
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
}

fn cmd_loop<F1, F2, F3, F4>(
    bots: &Bots,
    bot_instances: &mut BotInstances,
    tasks: &mut Tasks,
    mut get_input: F1,
    mut print_output: F2,
    mut print_error: F3,
    mut control_restart: F4,
) -> Result<(), String>
where
    F1: FnMut() -> Result<String, String>,
    F2: FnMut(&str) -> Result<(), String>,
    F3: FnMut(&str) -> Result<(), String>,
    F4: FnMut(&mut BotInstances) -> Result<(), String>,
{
    let mut task_serial_counter = 0;
    // start listening to stdin/control_bot for commands
    loop {
        let input = get_input()?;

        let parsed = cmd_parser::Cli::try_parse_from(
            "dcbothub"
                .split_whitespace()
                .chain(input.split_whitespace()),
        );

        let mut is_restart = false;

        let command_output = match &parsed {
            Ok(cli) => match &cli.command {
                cmd_parser::Commands::List => {
                    let mut output = String::new();
                    for name in bots.keys() {
                        output.push_str(&name);
                        output.push(' ');
                    }
                    let mut output = output.trim_end().to_string();
                    output.push('\n');
                    output
                }
                cmd_parser::Commands::ListStatus => {
                    let mut output = String::new();
                    for (name, instance) in bot_instances.iter_mut() {
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
                cmd_parser::Commands::ListTasks => {
                    let mut output = String::new();
                    for (id, ((bot_name, task_type, serial_number), instance)) in tasks.iter_mut() {
                        output.push_str(&format!(
                            "{id}\t{bot_name} {task_type} {serial_number} {} {}",
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
                cmd_parser::Commands::Status { bot_name } => {
                    match bot_instances.get_mut(bot_name) {
                        Some(instance) => {
                            format!(
                                "some {} {}\n",
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
                            )
                        }
                        None => "none\n".to_string(),
                    }
                }
                cmd_parser::Commands::TaskStatus { task_id } => match tasks.get_mut(task_id) {
                    Some(((bot_name, task_type, serial_number), instance)) => {
                        format!(
                            "some {task_id}\t{bot_name} {task_type} {serial_number} {} {}\n",
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
                        )
                    }
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Clean { bot_name } => match bots.get(bot_name) {
                    Some(bot) => {
                        if bot.has_repo() {
                            let task_id = format!("{:08}", task_serial_counter);
                            tasks.insert(
                                task_id.clone(),
                                (
                                    (bot_name.clone(), TaskType::Clean, task_serial_counter),
                                    bot.clean().unwrap()
                                        .stdin(std::process::Stdio::piped())
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::piped())
                                        .spawn()
                                        .map_err(|err| err.to_string()),
                                ),
                            );
                            task_serial_counter += 1;
                            format!("some {}\n", task_id)
                        } else {
                            "some no_repo\n".to_string()
                        }
                    }
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Build { bot_name } => match bots.get(bot_name) {
                    Some(bot) => {
                        if bot.has_repo() {
                            let task_id = format!("{:08}", task_serial_counter);
                            tasks.insert(
                                task_id.clone(),
                                (
                                    (bot_name.clone(), TaskType::Clean, task_serial_counter),
                                    bot.build().unwrap()
                                        .stdin(std::process::Stdio::piped())
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::piped())
                                        .spawn()
                                        .map_err(|err| err.to_string()),
                                ),
                            );
                            task_serial_counter += 1;
                            format!("some {}\n", task_id)
                        } else {
                            "some no_repo\n".to_string()
                        }
                    }
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Pull { bot_name } => match bots.get(bot_name) {
                    Some(bot) => {
                        if bot.has_repo() {
                            let task_id = format!("{:08}", task_serial_counter);
                            tasks.insert(
                                task_id.clone(),
                                (
                                    (bot_name.clone(), TaskType::Clean, task_serial_counter),
                                    bot.pull().unwrap()
                                        .stdin(std::process::Stdio::piped())
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::piped())
                                        .spawn()
                                        .map_err(|err| err.to_string()),
                                ),
                            );
                            task_serial_counter += 1;
                            format!("some {}\n", task_id)
                        } else {
                            "some no_repo\n".to_string()
                        }
                    }
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Start { bot_name } => {
                    if bot_instances.contains_key(bot_name) {
                        "exists\n".to_string()
                    } else {
                        match bots.get(bot_name) {
                            Some(bot) => {
                                bot_instances.insert(
                                    bot_name.clone(),
                                    bot.run()
                                        .stdin(std::process::Stdio::piped())
                                        .stdout(std::process::Stdio::piped())
                                        .stderr(std::process::Stdio::piped())
                                        .spawn()
                                        .map_err(|err| err.to_string()),
                                );
                                "none some spawned\n".to_string()
                            }
                            None => "none none\n".to_string(),
                        }
                    }
                }
                cmd_parser::Commands::Msg { bot_name, message } => {
                    match bot_instances.get_mut(bot_name) {
                        Some(Ok(child)) => match child.try_wait().unwrap() {
                            Some(_) => "started exited\n".to_string(),
                            None => {
                                let mut bot_out = BufWriter::new(child.stdin.as_mut().unwrap());
                                write!(bot_out, "{}\n", message.join(" ")).unwrap();
                                bot_out.flush().unwrap();
                                format!("started running written\n")
                            }
                        },
                        Some(Err(_)) => "failed\n".to_string(),
                        None => "none\n".to_string(),
                    }
                }
                cmd_parser::Commands::Verify { bot_name } => match bot_name {
                    Some(bot_name) => match bots.get(bot_name) {
                        Some(bot) => match bot.verify() {
                            Ok(_) => "some ok\n".to_string(),
                            Err(err) => format!("some err {}\n", err),
                        },
                        None => "none\n".to_string(),
                    },
                    None => {
                        let mut output = String::new();
                        for (bot_name, bot) in bots {
                            output.push_str(&match bot.verify() {
                                Ok(_) => format!("{} ok\n", bot_name),
                                Err(err) => format!("{} err {}\n", bot_name, err),
                            });
                        }
                        output
                    }
                },
                cmd_parser::Commands::Kill { bot_name } => match bot_instances.get_mut(bot_name) {
                    Some(Ok(child)) => match child.try_wait().unwrap() {
                        Some(_) => "started exited\n".to_string(),
                        None => {
                            child.kill().unwrap();
                            "started killed\n".to_string()
                        }
                    },
                    Some(Err(_)) => "failed\n".to_string(),
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::ControlRestart => {
                    control_restart(bot_instances)?;
                    is_restart = true;
                    "".to_string()
                }
                cmd_parser::Commands::Terminate { task_id } => match tasks.get_mut(task_id) {
                    Some((_, Ok(child))) => match child.try_wait().unwrap() {
                        Some(_) => "some started exited\n".to_string(),
                        None => {
                            child.kill().unwrap();
                            "some started killed\n".to_string()
                        }
                    },
                    Some((_, Err(_))) => "some failed\n".to_string(),
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Conclude { bot_name } => {
                    match bot_instances.get_mut(bot_name) {
                        Some(Ok(child)) => match child.try_wait().unwrap() {
                            Some(status) => {
                                let mut output = String::new();
                                output.push_str(&format!(
                                    "some started exited {}\n",
                                    status.code().unwrap_or(-1)
                                ));
                                let mut child_out = String::new();
                                child
                                    .stdout
                                    .take()
                                    .unwrap()
                                    .read_to_string(&mut child_out)
                                    .unwrap();
                                child_out.push('\n');
                                let mut child_err = String::new();
                                child
                                    .stderr
                                    .take()
                                    .unwrap()
                                    .read_to_string(&mut child_err)
                                    .unwrap();
                                child_err.push('\n');
                                output.push_str(&format!(
                                    "{} {} \n",
                                    child_out.lines().count(),
                                    child_err.lines().count()
                                ));
                                output.push_str(&child_out);
                                output.push_str(&child_err);
                                bot_instances.remove(bot_name);
                                output
                            }
                            None => "some started running\n".to_string(),
                        },
                        Some(Err(err)) => {
                            let output = format!("some failed {}\n", err);
                            bot_instances.remove(bot_name);
                            output
                        },
                        None => "none\n".to_string(),
                    }
                }
                cmd_parser::Commands::Wait { task_id } => match tasks.get_mut(task_id) {
                    Some((_, Ok(child))) => match child.try_wait().unwrap() {
                        Some(_) => "some started exited\n".to_string(),
                        None => {
                            child.wait().unwrap();
                            "some started waiting exited\n".to_string()
                        }
                    },
                    Some((_, Err(_))) => "some failed\n".to_string(),
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Finish { task_id } => match tasks.get_mut(task_id) {
                    Some((_, Ok(child))) => match child.try_wait().unwrap() {
                        Some(status) => {
                            let mut output = String::new();
                            output.push_str(&format!(
                                "some started exited {}\n",
                                status.code().unwrap_or(-1)
                            ));
                            let mut child_out = String::new();
                            child
                                .stdout
                                .take()
                                .unwrap()
                                .read_to_string(&mut child_out)
                                .unwrap();
                            child_out.push('\n');
                            let mut child_err = String::new();
                            child
                                .stderr
                                .take()
                                .unwrap()
                                .read_to_string(&mut child_err)
                                .unwrap();
                            child_err.push('\n');
                            output.push_str(&format!(
                                "{} {} \n",
                                child_out.lines().count(),
                                child_err.lines().count()
                            ));
                            output.push_str(&child_out);
                            output.push_str(&child_err);
                            tasks.remove(task_id);
                            output
                        }
                        None => "some started running\n".to_string(),
                    },
                    Some((_, Err(_))) => "some failed\n".to_string(),
                    None => "none\n".to_string(),
                },
                cmd_parser::Commands::Exit => {
                    break;
                }
            },
            Err(_) => "".to_string(),
        };

        if !is_restart {
            print_output(&command_output)?;
        }
        if parsed.is_err() {
            print_error(&format!("{}", parsed.unwrap_err()))?;
        }
    }
    Ok(())
}
