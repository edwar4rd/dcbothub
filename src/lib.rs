#[derive(Debug)]
pub struct Bot {
    name: String,
    repo_path: Option<std::path::PathBuf>,
    executable_path: std::path::PathBuf,
    url: Option<String>,
    build_args: Option<Vec<String>>,
    run_args: Option<Vec<String>>,
    token: Option<String>,
}

impl Bot {
    pub fn from_toml_table(table: &toml::value::Table) -> Result<Bot, String> {
        let name = match table.get("name") {
            Some(toml::Value::String(name)) => name.to_string(),
            Some(_) => {
                return Err("bot.name should be a string!".to_string());
            }
            None => {
                return Err("Given bot doesn't have a name!".to_string());
            }
        };

        let repo_path = match table.get("repo_path") {
            Some(toml::Value::String(path)) => {
                let path = std::path::Path::new(path).to_path_buf();
                match git2::Repository::open(&path) {
                    Ok(_) => Some(path),
                    Err(_) => {
                        return Err("bot.repo_path should lead to a git directiory!".to_string());
                    }
                }
            }
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
                if path.is_dir() || !path.exists() {
                    return Err(
                        "Given executable_path doesn't lead to a executable file!".to_string()
                    );
                }
                path
            }
            Some(_) => {
                return Err("bot.executable_path should be a string!".to_string());
            }
            None => {
                if let Some(repo_path) = &repo_path {
                    let path =
                        repo_path.join(std::path::Path::new(&format!("target/release/{}", name)));
                    if path.exists() {
                        path
                    } else {
                        return Err(
                            "repo_path/target/release/bot.name does't lead to a executable file!"
                                .to_string(),
                        );
                    }
                } else {
                    return Err("None of repo_path or executable_path is presented!".to_string());
                }
            }
        };

        let url = match table.get("url") {
            Some(toml::Value::String(url)) => match url::Url::parse(url) {
                Ok(_) => {
                    if repo_path.is_none() {
                        return Err(
                            "bot.url is presented although repo_path isn't!".to_string()
                        );
                    }
                    Some(url.to_string())
                }
                Err(_) => {
                    return Err("Given url cannot be parsed!".to_string());
                }
            },
            Some(_) => {
                return Err("bot.url should be a string!".to_string());
            }
            None => None,
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

        /*
        let build_args = match table.get("build_args") {
            Some(toml::Value::String(build_args)) => Some(build_args.as_str()),
            Some(_) => {
                return Err("bot.build_args should be a string!".to_string());
            }
            None => None,
        };

        let run_args = match table.get("run_args") {
            Some(toml::Value::String(run_args)) => Some(run_args.as_str()),
            Some(_) => {
                return Err("bot.run_args should be a string!".to_string());
            }
            None => None,
        };*/

        let token = match table.get("token") {
            Some(toml::Value::String(token)) => Some(token.to_string()),
            Some(_) => {
                return Err("bot.url should be a string!".to_string());
            }
            None => None,
        };

        Ok(Bot {
            name,
            repo_path,
            executable_path,
            url,
            build_args,
            run_args,
            token,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pull(&self) -> Result<std::process::Command, String> {
        match &self.repo_path {
            Some(repo_path) => {
                let mut command = std::process::Command::new("git");
                command.current_dir(repo_path).arg("pull");
                if let Some(url) = &self.url {
                    command.arg("--url").arg(url);
                }
                Ok(command)
            }
            None => {
                return Err("Target bot doesn't have a repo_path!".to_string());
            }
        }
    }

    pub fn rebuild(&self) -> Result<std::process::Command, String> {
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

    pub fn run(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.executable_path);
        if let Some(run_args) = &self.run_args {
            command.args(run_args);
        }
        if let Some(token) = &self.token {
            command.env("DISCORD_TOKEN", token);
        }
        command
    }
}
