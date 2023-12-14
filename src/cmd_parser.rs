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
    /// list name of all bots loaded from bots.toml in a line
    List,
    /// list every running/exited bot in a line
    ListExisting,
    /// list every running/exited task in a line
    ListExecuting,
    /// list every running/exited bot in a line with name and status
    ListStatus,
    /// list running/finished tasks such as build processes or pull processes
    ListTasks,
    /// get the status of a specific bot
    Status { bot_name: String },
    /// get the status of a specific task
    TaskStatus { task_id: String },
    /// perform a "cargo clean" at the repo of a bot without removing the executable
    Clean { bot_name: String },
    /// perform a "cargo clean" at the repo of a bot
    CleanAll { bot_name: String },
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
    /// verify all paths loaded from `bots.toml`,
    /// or only paths of the bot `BOT_NAME` if presented
    Verify { bot_name: Option<String> },
    /// stop a bot with the given name
    Kill { bot_name: String },
    /// kill the control bot, then attempt to restart it
    ControlRestart,
    /// stop a task with the given id
    Terminate { task_id: String },
    /// print out the exit status and output of a stopped bot and remove it from `bot_instances`
    Conclude { bot_name: String },
    /// blockingly wait a task to finish, or to fail, and return the exit status of the task
    Wait { task_id: String },
    /// print out the exit status and output of a finished/failed task and remove it from `tasks`
    Finish { task_id: String },
    /// kill all running tasks and bots and exit dcbothub
    Exit,
}
