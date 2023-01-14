# dcbothub

Bothub handle discord bots on a remote server for development purposes and can be controlled by a controller bot.

## Features

**MOST OF THE FEATURE AREN'T IMPLEMENTED YET!**
**ONLY THE CHECKED LINES ARE CURRENTLY IMPLEMENTED**

- [x] When started, bothub looks for `bots.toml` which contains paths and tokens for the bots.
- [x] By default, bothub starts all the listed bot as sub processes.
  - [ ] Command line flags can be set that bothub automatically build every bot on startup.   
- [x] Exactly one or none of the listed bot can be configurated as a controller bot, with which bothub communicates.
  - [x] When not presented, bothub uses stdin/stdout instead.
- [ ] Bothub can recieve commands to 
  - [ ] build a new executable for a bot.
  - [ ] stop a running bot instance and restart a new one.
- [ ] Bothub can automatically restart the controller bot if it stopped.
  - [ ] Different behavior can be configured with `bots.toml`
- [ ] Bothub can detect whether a bot has failed and activate a webhook accordingly.

## bots.toml

Every bots.toml file consists of the following sections:

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

## Program Structure

The aforementioned `bots.toml` is loaded once at the start of the program.
The program checks `bots.toml` and terminates early if it encounters any error.

Two separate checks are performed by the program:
  - A check analysis whether `bots.toml` is properly structured and include all necessary informations for the bots to be started.
    - This check evaluates to identical result for the same `bots.toml`, and only depends on the data within the file.
  - Another check validates that the paths presented in `bots.toml` actually exists in the file system.
    - The result of this check can be influenced by the invocation of commands.
    - A `verify` command can be invoked to perform this check afterwards.

The program then attempts to start every listed bot and waits for further instructions.

Three separate hash tables are used by the program:
  - `bots` represents the data loaded from `bots.toml`, and is never modified afterwards. 
    - however, it is possible for a previously valid path in `bots.toml` to become invalid, for example one can perform a `cargo clean` that make the executable path invalid.
  - `bot_instances` represents all attempts of starting bot, including successful and failed attempts.
    - Only one instance can be started for every bot, and failed attempt must be removed before a new one is started.
  - `tasks` represents all attempts of performing a task, including successful and failed attempts
    - Same type of task.

It is currently designed that most task related command just add a task to the `tasks` table, and the user can only check whether a task is finished, or wait for it to finish with the `wait` command.( Possible plan: trigger a command when a task is fininshed, but that'll probably require one to rewrite the whole thing into a async program. )

The program then loops indefinitely waiting for a command after the startup, until one of the following event occurs.
- A `exit` command is invoked.
- The program gets a `^C` or a `^D` from stdin.
- The program fails to communitate with `control_bot` (if presented in bots.toml), and all the automatic recovery attempts failed.

The program will perform the following to restart `control_bot` if it had terminated, and has a `restart_counter` storing how many times has the control bot been restarted (starts with zero) and a `last_restart` storing the last restart time:
- Remove the current `control_bot` bot instance and save its exit code and output to a separate log file.
  - If the exit code of `control_bot` isn't zero, `restart_counter` gets incremented by one.
  - If the exit code of `control_bot` is zero but the duration between `last_restart` and the current time is shorter than half a minute (30s), `restart_counter` gets incremented by one.
  - The saved file has the same format as the `conclude` command.
- If the `restart_counter` is lower than / equal to five, the program:
  - If the executable of `control_bot` is missing but has a `repo_path`, a `cargo clean` and a `cargo build` are performed.
  - If the executable of `control_bot` is still missing, and `control_bot` have no repo_path or either `cargo clean` or `cargo build` failed, the three below steps are skipped.
  - The program waits for 2^(`restart_counter+2`) seconds.
  - The program starts `control_bot` and add it to the `bot_instances` table.
  - The program stores the current time to `last_restart`.
- If the `restart_counter` is greater than five but lower than / equal to eight:
  - If `control_bot` has no `repo_path`, the program terminates.
  - If `control_bot` has a `repo_path`, a `git pull`, a `cargo clean` and a `cargo build` are performed.
  - If the executable of `control_bot` is (still) missing, and `control_bot` have no repo_path or either `git pull`, `cargo clean`, or `cargo build` failed, the three below steps are skipped.
  - The program waits for 2^(`restart_counter+2`) seconds.
  - The program starts `control_bot` and add it to the `bot_instances` table.
  - The program stores the current time to `last_restart`.
- If the `restart_counter` is greater than eight, the program terminates.

## Commands

These commands are shared by stdin input and control bot, and are potentially dangerous.

- [ ] `list [OPTIONS]` list name of all bots loaded from bots.toml each in a line
  - [ ] bots can be filtered out using options
  - since bots.toml is only loaded once in the startup of bothub, `list` should return the same results every time called, unless a status related option is included.
- [ ] `list-status [OPTIONS]` list every running/exited bot in a line with name and status listed
  - current format (of each line):
    - *BotName* (`started` (`running`|`exited` *ExitCode*))|(`failed` *FailureDescription*)
      - *ExitCode* is the exit code of exited bot as a decimal integer or -1 is it's terminated by a signal on unix
      - *FailureDescription* is a textual description related to how the bot failed starting with the specified executable
  - [ ] bots can be filtered out using options
- [ ] `list-tasks [OPTIONS]` list running/finished tasks such as build processes or pull processes
  - [ ] tasks can be filtered out using options
- None of the above commands guarantee a consistent order of the listing
- [ ] `status <BOT_NAME>` get the status of a specific bot_instance
  - current format (in a line):
    - (`none`|`some` *BotName* (`started` (`running`|`exited` *ExitCode*))|(`failed` *FailureDescription*))
      - *ExitCode* is the exit code of exited bot as a decimal integer or -1 is it's terminated by a signal on unix
      - *FailureDescription* is a textual description related to how the bot failed starting with the specified executable
- [ ] `task-status <TASK_ID>` get the status of a specific task
- [ ] `clean <BOT_NAME>` perform a `cargo clean` at the repo of a bot
  - subsequent `start` would fail if the executable is removed
- [ ] `build <BOT_NAME>` perform a `cargo build` at the repo of a bot
  - executable file would not be updated is cargo couldn't compile the executable
- [ ] `pull <BOT_NAME>` perform a `git pull` at the repo of a bot
- the three above commands start a process as a task of dcbothub
- output of the commands only indicates whether the task is started succesfully, and the assigned task id
- [ ] `start <BOT_NAME>` start the bot if it isn't already runninng
- [ ] `msg <BOT_NAME> [MESSAGE]...` print a message to the stdin of the a bot
- [ ] `verify [BOT_NAME]` verify all paths loaded from `bots.toml`, or only paths of the bot `BOT_NAME` if presented.
- [ ] `kill <BOT_NAME>` stop a bot with the given name
  - by sending a SIGKILL on *nix
  - killing `control_bot` actives the aforementioned auto-recovery process of dcbothub
  - to actually stop the program, the control bot should first gracefully shutdown itself then call the `exit` command
- [ ] `control-restart` kill the control bot, then attempt to restart it
  - a failed attempt to start the bot activates auto-recovery process
- [ ] `terminate <TASK_ID>` stop a task with the given id
- [ ] `conclude <BOT_NAME>` print out the exit status and output of a stopped bot and remove it from `bot_instances`
- [ ] `wait <TASK_ID>` blockingly wait a task to finish, or to fail, and return the exit status of the task
  - during the wait, the program wouldn't respond to any commands, and it is currently impossible to cancel the wait
- [ ] `finish <TASK_ID>` print out the exit status and output of a finished/failed task and remove it from `tasks`
- the output of `conclude` and `finish` command is in the same format, first the line counts of stdout and stderr separated by a space, then stdout, then stderr.
  - bothub append a line after both stdour content and stderr content
- [ ] `exit` kill all running tasks and bots, then exit dcbothub

When running with a control_bot, dcbothub adds a line of one integer indicating how many line does the command output span.

For example, running the dcbothub with the above example and assuming all paths are valid,
if the bot sends `list\n`,
dcbothub replies with `1\nbot_a bot_b\n`.
This helps control_bot deals with multiline replies.
