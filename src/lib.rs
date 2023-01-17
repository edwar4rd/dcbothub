pub mod bot_parser;
pub mod cmd_parser;

#[derive(std::fmt::Debug)]
pub enum TaskType {
    Clean,
    Build,
    Pull,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Clean => "Clean",
                Self::Build => "Build",
                Self::Pull => "Pull",
            }
        )
    }
}

pub type Bots = std::collections::HashMap<String, bot_parser::Bot>;
pub type BotInstances = std::collections::HashMap<String, Result<std::process::Child, String>>;
pub type Tasks = std::collections::HashMap<
    String,
    ((String, TaskType, u32), Result<std::process::Child, String>),
>;
