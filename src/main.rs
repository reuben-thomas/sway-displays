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

    let user_command = cli();
    let config_file_path = user_command
        .config_file_path
        .unwrap_or(Settings::default_config_file_path());
    let mut settings = Settings::load_from_file(&config_file_path);

    match user_command.action {
        Action::List => {
            println!("{}", settings.to_yaml());
        }
        Action::Save => {
            settings.default_configurations.insert(
                DefaultConfigIdentifier::from(&outputs),
                Config::from_sway_outputs_workspaces(&outputs, &workspaces),
            );
            settings.save_to_file(&config_file_path);
        }
        Action::SaveCustom(custom_config_name) => {
            settings.custom_configurations.insert(
                CustomConfigIdentfier(custom_config_name),
                Config::from_sway_outputs_workspaces(&outputs, &workspaces),
            );
            settings.save_to_file(&config_file_path);
        }
        Action::Set => {
            let current_config_identifier = DefaultConfigIdentifier::from(&outputs);
            if let Some(current_config) = settings
                .default_configurations
                .get(&current_config_identifier)
            {
                current_config.set_in_sway(&outputs, &mut connection);
                println!(
                    "Set default configuration for {:?}",
                    current_config_identifier
                );
            } else {
                println!(
                    "You don't already have a default configuration saved for {:?}",
                    current_config_identifier
                );
            }
        }
        _ => println!("Either it doesn't work, or you typed the wrong thing"),
    }
}
