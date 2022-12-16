# dcbothub

Bothub handle the summation of different discord bot on a server and can be controlled by a controller bot.

## Features

**MOST OF THE FEATURE AREN'T IMPLEMENTED YET!**
**ONLY THE CHECKED LINES ARE CURRENTLY IMPLEMENTED**

- [x] When started, bothub looks for `bots.toml` which contain paths to bot repo and tokens for the bots.
- [x] By default, bothub starts all the listed bot as sub processes.
- [ ] Exactly one or none of the listed bot can be a controller bot, which bothub communicates to and reacts accordingly.
  - [ ] When not presented, bothub uses stdin/stdout instead.
- [ ] Bothub can recieve command to 
  - [ ] `git pull` and `cargo build` a new executable for a bot
  - [ ] Kill the running bot and restart a new one
  - [ ] Detect whether one of the bot have failed and activate a webhook accordingly

## bots.toml

Every bots.toml file consists of the following sections

- `control_bot`: a string which is the name of a bot in `bot` the array.
- `bot`: a array of table, each table describes a bot to be runned by bothub.
  - `name`: a string that unique identify a bot (required)
    - including a whitespace or newline in the bot name is forbiddened since that will probably break something
  - `repo_path`: a string that locates a path to a cargo/git repository that contains the bot source code
    - if `repo_path` isn't presented in the table, doing a rebuild for the bot will fail
  - `executable_path`: bothub looks for the executable of the bot by default in `repo_path/target/release/bot_name`, if that's not the correct path then a `executable_path` is required
    - if both `repo_path` and `executable_path` is presented, `executable_path` is prefered over `repo_path`
      - if `executable_path` is a relative path, it is treated as related to `executable_path` 
  - atleast one of the two above value must be specified in a `bot` table
    - i.e. the program must be able to infer a executable path
  - `url`: a string that a url for bothub to do `git pull url` from
    - this value should only present if `repo_path` is presented
  - `build_args`: a array of string that is passed to cargo when running `cargo build args`
    - by default, bothub do `cargo build --release` when a rebuild is requested
    - this value should only present if `repo_path` is presented
  - `run_args`: a array of string that is passed to the executable (not cargo!) when running
  - `token`: a string that's requested from discord application website that can be used to authenticate the bot when establishing a gateway connection
    - environment variable `DISCORD_TOKEN` is set to `token` for the bot

Here's a example `bots.toml` file

```toml
control_bot="bot_a"

[[bot]]
name="bot_a"
repo_path = "~/path/to/repo"
url = "https://alternative.origin.to/pull/from"
build_args = ["--release","--all-features"]
token = "MTA0IAMNOTGIVINGYOUMYDiscoRd.BotTokEN.Liketh1sLo1D0nTC0pYthi5AndP4sTeIt"

[[bot]]
name="bot_b"
executable_path = "~/path/to/somewhere/else/bin/bot_a"
run_args = ["--silent", "--no-cache", "--database-dir=/some/more/path"]
token = "MTA0IAMNOTGIVINGYOUMYDiscoRd.BotTokEN.BoTsDoNtSh4r3T0keNsBTWJusTpAdd1n6"
```

## Commands

These commands are shared by stdin input and control bot, and are potentially dangerous.

- [ ] `list [OPTIONS]` list name of all bots loaded from bots.toml each in a line
  - [ ] bots can be filtered out using options
- [ ] `list-status [OPTIONS]` list every running/exited bot in a line with name and status listed
  - [ ] bots can be filtered out using options
- [ ] `list-tasks [OPTIONS]` list running/finished tasks such as build processes or pull processes
  - [ ] tasks can be filtered out using options
- [ ] `status <BOT_NAME>` get the status of a specific bot
- [ ] `task-status <TASK_ID>` get the status of a specific task
- [ ] `clean <BOT_NAME>` perform a `cargo clean` at the repo of a bot
  - subsequent `start` would fail if the executable is removed
- [ ] `build <BOT_NAME>` perform a `cargo build` at the repo of a bot
  - executable file would not be updated is cargo couldn't compile the executable
- [ ] `pull <BOT_NAME>` perform a `git pull` at the repo of a bot
- the three above process are started as a task of dcbothub
- output of the commands only indicates whether the task is started succesfully
- [ ] `start <BOT_NAME>` start the bot if it isn't already runninng
- [ ] `msg <BOT_NAME> [MESSAGE]...` print a message to the stdin of the a bot
- [ ] `kill <BOT_NAME>` stop a bot with the given name
  - by sending a SIGKILL on *nix
- [ ] `exit` stop all running tasks and bots and exit dcbothub