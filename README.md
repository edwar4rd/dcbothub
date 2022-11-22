# dcbothub

Bothub handle the summation of different discord bot on a server and can be controlled by a controller bot.

## Features

**MOST OF THE FEATURE AREN'T IMPLEMENTED YET!**
**ONLY THE CHECKED LINES ARE CURRENTLY IMPLEMENTED**

- [ ] When started, bothub looks for BOTS file containing paths to bot repo and tokens for the bots.
- [ ] By default, bothub starts all the listed bot as sub processes.
- [ ] The first listed bot is treated as a controller bot (by default), which bothub communicate to and react accordingly.
  - [ ] Or bothub can be configure to use stdin/stdout instead.
- [ ] Bothub can recieve command to
  - [ ] `git pull` and `cargo build` a new executable for a bot
  - [ ] Kill the running bot and restart a new one
  - [ ] Detect whether one of the bot have failed and activate a webhook accordingly
