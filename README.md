# dcbothub

Bothub handle the summation of different discord bot on a server and can be controlled by a controller bot.

## Features

**MOST OF THE FEATURE AREN'T IMPLEMENTED YET!**
**ONLY THE CHECKED LINES ARE CURRENTLY IMPLEMENTED**

- [ ] When started, bothub looks for `bots.toml` which contain paths to bot repo and tokens for the bots.
- [ ] By default, bothub starts all the listed bot as sub processes.
- [ ] Exactly one or none of the listed bot can be a controller bot, which bothub communicates to and reacts accordingly.
  - [ ] When not presented, bothub uses stdin/stdout instead.
- [ ] Bothub can recieve command to 
  - [ ] `git pull` and `cargo build` a new executable for a bot
  - [ ] Kill the running bot and restart a new one
  - [ ] Detect whether one of the bot have failed and activate a webhook accordingly

## bots.toml

Every bots.toml file consists of the following sections

- `control_bot`
- bot

Here's a example `bots.toml` file

```toml
control_bot=""

[bots.a_bot]
repo_path = "~/path/to/repo"
token = "MTA0IAMNOTGIVINGYOUMYDiscoRd.BotTokEN.Liketh1sLo1D0nTC0pYthi5AndP4sTeIt"
execuatable_path = "~/path/to/repo/target/release/bot_a"

[bots.another]
repo_path = "~/path/to/somewhere/else"
token = "MTA0IAMNOTGIVINGYOUMYDiscoRd.BotTokEN.BoTsDoNtSh4r3T0keNsBTWJusTpAdd1n6"
url = "https://alternative.origin.to/pull/from"
execuatable_path = "~/path/to/somewhere/else/target/release/bot_a"
```

## Commands

These commands are shared by stdin input and control bot, and are potentially dangerous.

- [ ] `list [options]` list all bot to output and their status
