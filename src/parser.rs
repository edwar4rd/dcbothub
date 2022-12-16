use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(subcommand_required = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// list name of all bots loaded from bots.toml each in a line
    List,
    /// list every running/exited bot in a line with name and status listed
    ListStatus,
    /// list running/finished tasks such as build processes or pull processes
    ListTasks,
    /// get the status of a specific bot
    Status { bot_name: String },
    /// get the status of a specific task
    TaskStatus { task_id: String },
    /// perform a "cargo clean" at the repo of a bot
    Clean { bot_name: String },
    /// perform a "cargo build" at the repo of a bot
    Build { bot_name: String },
    /// perform a "git pull" at the repo of a bot
    Pull { bot_name: String },
    /// start the bot if it isn't already runninng
    Start { bot_name: String },
    /// print a message to the stdin of the a bot
    Msg {
        bot_name: String,
        #[arg(action = clap::ArgAction::Append)]
        message: Vec<String>,
    },
    /// stop a bot with the given name
    Kill { bot_name: String },
    /// kill all running tasks and bots and exit dcbothub
    Exit,
}
