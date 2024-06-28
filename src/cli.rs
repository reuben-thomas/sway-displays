use std::path::PathBuf;
use clap::{arg, Command};

#[derive(Debug, Default, Clone)]
pub struct UserCommand {
    pub action: Action,
pub config_file_path: Option<PathBuf>,
}

#[derive(Debug, Default, Clone)]
pub enum Action {
    List,
    Save,
    SaveCustom(String),
    Set,
    SetCustom(String),
    RunContinuous,
    #[default]
    Invalid,
}

pub fn cli() -> UserCommand {
    let arg_matches = Command::new("sway-displays")
        .about("A tool to manage display configurations in Sway.\n\n\
            Default configurations are saved and can then be loaded based on the connected displays.\n\
            Custom configurations are saved and loaded by name. There can be multiple custom configurations for a set connected displays.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("list").about("List all saved configurations"))
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

    match arg_matches.subcommand() {
        Some(("list", _)) => UserCommand {
            action: Action::List,
            config_file_path,
        },
        Some(("save", _)) => UserCommand {
            action: Action::Save,
            config_file_path,
        },
        Some(("save-custom", sub_matches)) => UserCommand {
            action: Action::SaveCustom(
                sub_matches
                    .get_one::<String>("custom_config_name")
                    .expect("Missing custom config name")
                    .to_string(),
            ),
            config_file_path,
        },
        Some(("set", _)) => UserCommand {
            action: Action::Set,
            config_file_path,
        },
        Some(("set-custom", sub_matches)) => UserCommand {
            action: Action::SaveCustom(
                sub_matches
                    .get_one::<String>("custom_config_name")
                    .expect("Missing custom config name")
                    .to_string(),
            ),
            config_file_path,
        },
        Some(("run", _)) => UserCommand {
            action: Action::RunContinuous,
            config_file_path,
        },
        _ => UserCommand::default(),
    }
}
