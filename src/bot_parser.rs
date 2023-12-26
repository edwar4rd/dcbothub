#[derive(Debug)]
pub struct Bot {
    name: String,
    repo_path: Option<std::path::PathBuf>,
    executable_path: std::path::PathBuf,
    build_args: Option<Vec<String>>,
    run_args: Option<Vec<String>>,
    token: Option<String>,
}

impl Bot {
    fn from_toml_table(table: &toml::value::Table) -> Result<Bot, String> {
        let name = match table.get("name") {
            Some(toml::Value::String(name)) => {
                if name.contains(char::is_whitespace) {
                    return Err("bot.name should contain no whitespace!".to_string());
                }
                name.to_string()
            }
            Some(_) => {
                return Err("bot.name should be a string!".to_string());
            }
            None => {
                return Err("Given bot doesn't have a name!".to_string());
            }
        };

        let repo_path = match table.get("repo_path") {
            Some(toml::Value::String(path)) => Some(
                match std::path::Path::new(path).to_path_buf().canonicalize() {
                    Ok(path) => path,
                    Err(_) => {
                        return Err("bot.repo_path should be resolvable to a directory!".into())
                    }
                },
            ),
            Some(_) => {
                return Err("bot.repo_path should be a string!".to_string());
            }
            None => None,
        };

        let executable_path = match table.get("executable_path") {
            Some(toml::Value::String(path)) => {
                let mut path = std::path::Path::new(path).to_path_buf();
                if let Some(repo_path) = &repo_path {
                    if path.is_relative() {
                        let mut new_path = repo_path.clone();
                        new_path.push(path);
                        path = new_path.to_path_buf();
                    }
                }
                path
            }
            Some(_) => {
                return Err("bot.executable_path should be a string!".to_string());
            }
            None => {
                if let Some(repo_path) = &repo_path {
                    repo_path.join(std::path::Path::new(&format!("target/release/{}", name)))
                } else {
                    return Err("None of repo_path or executable_path is presented!".to_string());
                }
            }
        };

        let build_args = match table.get("build_args") {
            Some(toml::Value::Array(arr)) => {
                if repo_path.is_none() {
                    return Err("bot.build_args is presented although repo_path isn't!".to_string());
                }
                let mut args: Vec<String> = Vec::new();
                for arg in arr {
                    if let toml::Value::String(arg) = arg {
                        args.push(arg.to_string());
                    } else {
                        return Err("element of bot.build_args should be a string!".to_string());
                    }
                }
                Some(args)
            }
            Some(_) => {
                return Err("bot.build_args should be a array!".to_string());
            }
            None => None,
        };

        let run_args = match table.get("run_args") {
            Some(toml::Value::Array(arr)) => {
                let mut args: Vec<String> = Vec::new();
                for arg in arr {
                    if let toml::Value::String(arg) = arg {
                        args.push(arg.to_string());
                    } else {
                        return Err("element of bot.run_args should be a string!".to_string());
                    }
                }
                Some(args)
            }
            Some(_) => {
                return Err("bot.run_args should be a array!".to_string());
            }
            None => None,
        };

        let token = match table.get("token") {
            Some(toml::Value::String(token)) => Some(token.to_string()),
            Some(_) => {
                return Err("bot.token should be a string!".to_string());
            }
            None => None,
        };

        Ok(Bot {
            name,
            repo_path,
            executable_path,
            build_args,
            run_args,
            token,
        })
    }

    /// checks if repo_path(if specified) and inferred executable_path actaully exists on the file system
    pub fn verify(&self) -> Result<(), String> {
        match &self.repo_path {
            Some(path) => match git2::Repository::open(path) {
                Ok(_) => {}
                Err(_) => {
                    return Err("bot.repo_path should lead to a git directiory!".to_string());
                }
            },
            None => {}
        };

        if !self.executable_path.is_file() {
            return Err("Given executable_path doesn't lead to a executable file!".to_string());
        }

        return Ok(());
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn has_repo(&self) -> bool {
        self.repo_path.is_some()
    }

    pub fn clean(&self) -> Result<std::process::Command, String> {
        let executable_path = match self.executable_path.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                return Err(
                    "Failed resolving repo/executable relation, use cleanall instead!".into(),
                )
            }
        };
        let repo_path = match &self.repo_path {
            Some(path) => match path.canonicalize() {
                Ok(path) => path,
                Err(_) => {
                    return Err(
                        "Failed resolving repo/executable relation, use cleanall instead!".into(),
                    )
                }
            },
            None => {
                return Err("Target bot doesn't have a repo_path!".to_string());
            }
        };

        if executable_path.starts_with(&repo_path) {
            let mut command = std::process::Command::new("bash");
            command.current_dir(repo_path).arg("-c").arg(format!("(export EXEC_PATH={}; (export DCBOTHUB_TMP_EXEC_PATH=\"dcbothub_tmp_exec_$(echo $RANDOM)\"; mv $EXEC_PATH $DCBOTHUB_TMP_EXEC_PATH && cargo clean && mkdir -p \"$(dirname $EXEC_PATH)\" && mv $DCBOTHUB_TMP_EXEC_PATH $EXEC_PATH;))",self.executable_path.display()));
            Ok(command)
        } else {
            let mut command = std::process::Command::new("cargo");
            command.current_dir(repo_path).arg("clean");
            Ok(command)
        }
    }

    pub fn clean_all(&self) -> Result<std::process::Command, String> {
        match &self.repo_path {
            Some(repo_path) => {
                let mut command = std::process::Command::new("cargo");
                command.current_dir(repo_path).arg("clean");
                Ok(command)
            }
            None => {
                return Err("Target bot doesn't have a repo_path!".to_string());
            }
        }
    }

    pub fn build(&self) -> Result<std::process::Command, String> {
        match &self.repo_path {
            Some(repo_path) => {
                let mut command = std::process::Command::new("cargo");
                command.current_dir(repo_path).arg("build");
                if let Some(build_args) = &self.build_args {
                    command.args(build_args);
                } else {
                    command.arg("--release");
                }
                Ok(command)
            }
            None => {
                return Err("Target bot doesn't have a repo_path!".to_string());
            }
        }
    }

    pub fn pull(&self) -> Result<std::process::Command, String> {
        match &self.repo_path {
            Some(repo_path) => {
                let mut command = std::process::Command::new("git");
                command.current_dir(repo_path).arg("pull");
                Ok(command)
            }
            None => {
                return Err("Target bot doesn't have a repo_path!".to_string());
            }
        }
    }

    pub fn run(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.executable_path);
        if let Some(repo_path) = &self.repo_path {
            command.current_dir(repo_path);
        }
        if let Some(run_args) = &self.run_args {
            command.args(run_args);
        }
        if let Some(token) = &self.token {
            command.env("DISCORD_TOKEN", token);
        }
        command
    }
}

use toml;
pub fn parse_bots() -> Result<(std::collections::HashMap<String, Bot>, Option<String>), ()> {
    let file = match std::fs::read_to_string("bots.toml") {
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

    let mut hashmap = std::collections::HashMap::new();

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

        match bot.verify() {
            Ok(_) => {}
            Err(err) => {
                println!("Failed on verifying paths for {}:\n\t{}", bot.name(), err);
                return Err(());
            }
        }

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
