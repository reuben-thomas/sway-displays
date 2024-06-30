use swayipc::Connection;

pub mod cli;
pub use cli::*;

pub mod settings;
pub use settings::*;

fn main() {
    let mut connection = Connection::new().expect("Failed to connect to Sway IPC");
    let workspaces = connection
        .get_workspaces()
        .expect("Failed to get workspaces");
    let outputs = connection.get_outputs().expect("Failed to get outputs");

    let user_command = cli::user_command();
    let config_file_path = user_command
        .config_file_path
        .unwrap_or(Settings::default_config_file_path());
    let mut settings = Settings::load_from_file(&config_file_path);

    match user_command.action {
        Action::List => {
            println!(
                "{}",
                serde_yaml::to_string(&settings).expect("Unable to serialize settings")
            );
        }
        Action::ShowConnected => {
            println!(
                "Connected outputs:\n{}",
                DefaultConfigIdentifier::from(&outputs)
            );
        }
        Action::Save => {
            let current_config_identifier = DefaultConfigIdentifier::from(&outputs);
            let current_config = Config::from_sway_outputs_workspaces(&outputs, &workspaces);
            if settings
                .default_configurations
                .contains_key(&current_config_identifier)
            {
                if !cli::confirm_overwrite(&current_config_identifier.to_string()) {
                    return;
                }
            }
            settings
                .default_configurations
                .insert(current_config_identifier.clone(), current_config);
            settings.save_to_file(&config_file_path);
            println!(
                "Saved default configuration for outputs:\n{}",
                current_config_identifier
            );
        }
        Action::SaveCustom(custom_config_name) => {
            let current_config_identifier = CustomConfigIdentfier(custom_config_name);
            let current_config = Config::from_sway_outputs_workspaces(&outputs, &workspaces);
            if settings
                .custom_configurations
                .contains_key(&current_config_identifier)
            {
                if !cli::confirm_overwrite(&current_config_identifier.to_string()) {
                    return;
                }
            }
            settings
                .custom_configurations
                .insert(current_config_identifier.clone(), current_config);
            settings.save_to_file(&config_file_path);
            println!(
                "Saved custom configuration named: {}",
                current_config_identifier
            );
        }
        Action::Set => {
            let current_config_identifier = DefaultConfigIdentifier::from(&outputs);
            if let Some(current_config) = settings
                .default_configurations
                .get(&current_config_identifier)
            {
                current_config.set_in_sway(&outputs, &mut connection);
                println!(
                    "Setting default configuration for outputs:\n{}",
                    current_config_identifier
                );
            } else {
                println!(
                    "No existing configuration found for outputs:\n{}",
                    current_config_identifier
                )
            }
        }
        Action::SetCustom(custom_config_name) => {
            if let Some(custom_config) = settings
                .custom_configurations
                .get(&CustomConfigIdentfier(custom_config_name.clone()))
            {
                custom_config.set_in_sway(&outputs, &mut connection);
                println!("Setting custom configuration: {}", custom_config_name);
            } else {
                println!(
                    "No existing custom configuration named: {}",
                    custom_config_name
                );
            }
        }
        Action::RunContinuous => {
            println!("Exiting continuous mode");
        }
        _ => println!("Invalid command. Run with --help for usage information."),
    }
}
