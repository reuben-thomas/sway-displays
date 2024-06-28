use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use swayipc::{Connection, Output as SwayOutput, Workspace as SwayWorkspace};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct OutputProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<String>>,
}

impl OutputProperties {
    pub fn to_sway_output_command(&self, connection_name: &String) -> String {
        let mut command = "output ".to_string() + connection_name;
        if let Some(active) = self.active {
            if active {
                command.push_str(" enable");
            } else {
                command.push_str(" disable");
            }
        }
        if self.resolution.is_some() && self.refresh_rate.is_some() {
            command.push_str(&format!(
                " mode {}x{}@{}Hz",
                self.resolution.unwrap().0,
                self.resolution.unwrap().1,
                self.refresh_rate.unwrap()
            ));
        } else if let Some((width, height)) = self.resolution {
            command.push_str(&format!(" res {}x{}", width, height));
        }
        if let Some((x, y)) = self.position {
            command.push_str(&format!(" pos {} {}", x, y));
        }
        if let Some(rotation) = &self.rotation {
            command.push_str(&format!(" transform {}", rotation));
        }
        if let Some(scale) = self.scale {
            command.push_str(&format!(" scale {:2}", scale));
        }
        return command;
    }

    pub fn to_sway_workspace_command(&self, connection_name: &String) -> String {
        let mut commands = Vec::<String>::new();
        if let Some(workspaces) = &self.workspaces {
            for workspace in workspaces {
                commands.push(format!("workspace {} output {}", workspace, connection_name));
            }
        }
        return commands.join(";");
    }
}

impl From<&SwayOutput> for OutputProperties {
    fn from(output: &SwayOutput) -> Self {
        OutputProperties {
            active: Some(output.active),
            resolution: if output.rect.width > 0 && output.rect.height > 0 {
                Some((output.rect.width, output.rect.height))
            } else {
                None
            },
            position: if output.rect.x > 0 && output.rect.y > 0 {
                Some((output.rect.x, output.rect.y))
            } else {
                None
            },
            rotation: if output.transform.as_ref().is_some_and(|t| t != "normal") {
                output.transform.clone()
            } else {
                None
            },
            scale: if output.scale.is_some_and(|scale| (1.0 - scale).abs() > 0.01) {
                output.scale
            } else {
                None
            },
            refresh_rate: output.current_mode.and_then(|mode| Some(mode.refresh)),
            workspaces: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
pub struct OutputIdentifier(String);

impl From<&SwayOutput> for OutputIdentifier {
    fn from(output: &SwayOutput) -> Self {
        OutputIdentifier(format!(
            "{} {} {}",
            output.make, output.model, output.serial
        ))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Config(HashMap<OutputIdentifier, OutputProperties>);

impl Config {
    pub fn from_sway_outputs_workspaces(
        outputs: &Vec<SwayOutput>,
        workspaces: &Vec<SwayWorkspace>,
    ) -> Self {
        let mut new_config = Config::default();
        let mut output_workspace_map: HashMap<String, Vec<String>> = HashMap::new();
        for workspace in workspaces {
            output_workspace_map
                .entry(workspace.output.clone())
                .and_modify(|v| v.push(workspace.name.clone()))
                .or_insert(vec![workspace.name.clone()]);
        }
        for output in outputs {
            let output_identifier = OutputIdentifier::from(output);
            let mut output_properties = OutputProperties::from(output);
            if let Some(workspaces) = output_workspace_map.get(&output.name) {
                if workspaces.len() > 0 {
                    output_properties.workspaces = Some(workspaces.clone());
                }
            }
            new_config.0.insert(output_identifier, output_properties);
        }
        return new_config;
    }

    pub fn set_in_sway(&self, outputs: &Vec<SwayOutput>, connection: &mut Connection) {
        let mut commands: Vec<String> = Vec::new();
        for output in outputs {
            if let Some(output_properties) = self.0.get(&OutputIdentifier::from(output)) {
                commands.push(output_properties.to_sway_output_command(&output.name));
                commands.push(output_properties.to_sway_workspace_command(&output.name));
            }
        }
        let commands = commands.join(";\n");
        println!("Running command: {}", commands);
        connection
            .run_command(&commands)
            .expect("Failed to run command");
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
pub struct DefaultConfigIdentifier(Vec<OutputIdentifier>);

impl From<&Vec<SwayOutput>> for DefaultConfigIdentifier {
    fn from(outputs: &Vec<SwayOutput>) -> Self {
        DefaultConfigIdentifier(
            outputs
                .iter()
                .map(|output| OutputIdentifier::from(output))
                .collect(),
        )
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
pub struct CustomConfigIdentfier(pub String);

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom_configurations: HashMap<CustomConfigIdentfier, Config>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub default_configurations: HashMap<DefaultConfigIdentifier, Config>,
}

impl Settings {
    pub fn load_from_file(config_file_path: &PathBuf) -> Settings {
        let config_file =
            std::fs::read_to_string(config_file_path).expect("Unable to read config file");
        return serde_yaml::from_str(&config_file).expect("Unable to parse config file");
    }

    pub fn save_to_file(&self, config_file_path: &PathBuf) {
        let config_file = serde_yaml::to_string(&self).expect("Unable to serialize settings");
        std::fs::write(config_file_path, config_file).expect("Unable to write config file");
    }

    pub fn default_config_file_path() -> PathBuf {
        config_dir()
            .expect("Unable to to get default config directory")
            .join("sway-displays/config.yml")
    }

    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(&self).expect("Unable to serialize settings")
    }
}
