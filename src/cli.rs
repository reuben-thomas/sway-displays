use clap::{arg, Command};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct UserCommand {
    pub action: Action,
    pub config_file_path: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
pub enum Action {
    List,
    ShowConnected,
    Save,
    SaveCustom(String),
    Set,
    SetCustom(String),
    RunContinuous,
    #[default]
    Invalid,
}

pub fn user_command() -> UserCommand {
    let arg_matches = Command::new("sway-displays")
        .about("A tool to manage display configurations in Sway.\n\n\
            Default configurations are saved and can then be loaded based on the connected displays.\n\
            Custom configurations are saved and loaded by name. There can be multiple custom configurations for a set connected displays.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("list").about("List all saved configurations"))
        .subcommand(Command::new("show-connected").about("Show names of connected displays"))
        .subcommand(
            Command::new("save").about("Save current as a default configuration"),
        )
        .subcommand(
            Command::new("save-custom")
                .about("Save current layout as a custom configuration")
                .arg(arg!(custom_config_name: [CUSTOM_CONFIG_NAME]))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("set")
                .about("Automatically set a default configuration based on connected displays"),
        )
        .subcommand(
            Command::new("set-custom")
                .about("Set a custom configuration by name")
                .arg(arg!(custom_config_name: [CUSTOM_CONFIG_NAME]))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("run")
                .about("Run in continuous mode and automatically set apply default configurations based on connected displays")
                .arg(arg!(custom_config_name: [CUSTOM_CONFIG_NAME]))
                .arg_required_else_help(true),
        )
        .arg(
            arg!(-c --config <CONFIG_FILE_PATH> "Use a custom config file")
                .required(false)
                .global(true)
        )
        .get_matches();

    let config_file_path = arg_matches
        .get_one::<String>("config")
        .map(|s| PathBuf::from(s.to_string()));

    UserCommand {
        action: match arg_matches.subcommand() {
            Some(("list", _)) => Action::List,
            Some(("show-connected", _)) => Action::ShowConnected,
            Some(("save", _)) => Action::Save,
            Some(("save-custom", sub_matches)) => Action::SaveCustom(
                sub_matches
                    .get_one::<String>("custom_config_name")
                    .expect("Missing custom config name")
                    .to_string(),
            ),
            Some(("set", _)) => Action::Set,
            Some(("set-custom", sub_matches)) => Action::SetCustom(
                sub_matches
                    .get_one::<String>("custom_config_name")
                    .expect("Missing custom config name")
                    .to_string(),
            ),
            Some(("run", _)) => Action::RunContinuous,
            _ => Action::Invalid,
        },
        config_file_path,
    }
}

pub fn confirm_overwrite(config_name: &String) -> bool {
    let confirmation = format!(
        "There already exists a configuration {}\nOverwrite? (y/n)",
        config_name
    );
    println!("{}", confirmation);

    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let input = input.trim().to_lowercase();
            input == "y" || input == "yes"
        }
        Err(_) => false,
    }
}
