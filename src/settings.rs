use dirs::config_dir;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, fmt::Display, path::PathBuf};
use swayipc::{Connection, Output as SwayOutput, Workspace as SwayWorkspace};

#[derive(Clone, Serialize, Deserialize, Default)]
struct OutputProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolution: Option<(i32, i32)>,
    position: (i32, i32),
    #[serde(skip_serializing_if = "Option::is_none")]
    rotation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspaces: Option<Vec<String>>,
}

impl OutputProperties {
    fn to_sway_output_command(&self, connection_name: &str) -> String {
        let mut command = "output ".to_string() + connection_name;
        if let Some(active) = self.active {
            if active {
                command.push_str(" enable");
            } else {
                command.push_str(" disable");
            }
        }
        if let (Some(resolution), Some(refresh_rate)) = (self.resolution, self.refresh_rate) {
            command.push_str(&format!(
                " mode {}x{}@{:.3}Hz",
                resolution.0,
                resolution.1,
                refresh_rate as f64 / 1000.0
            ));
        } else if let Some((width, height)) = self.resolution {
            command.push_str(&format!(" res {}x{}", width, height));
        }
        let (x, y) = self.position;
        command.push_str(&format!(" pos {} {}", x, y));
        if let Some(rotation) = &self.rotation {
            command.push_str(&format!(" transform {}", rotation));
        }
        if let Some(scale) = self.scale {
            command.push_str(&format!(" scale {:2}", scale));
        }
        command
    }

    fn to_sway_workspace_command(&self, connection_name: &str) -> String {
        let mut commands = Vec::<String>::new();
        if let Some(workspaces) = &self.workspaces {
            for workspace in workspaces {
                commands.push(format!(
                    "workspace {} output {}",
                    workspace, connection_name
                ));
            }
        }
        commands.join(";")
    }
}

impl From<&SwayOutput> for OutputProperties {
    fn from(output: &SwayOutput) -> Self {
        OutputProperties {
            active: Some(output.active),
            resolution: if output.rect.width > 0 || output.rect.height > 0 {
                Some((output.rect.width, output.rect.height))
            } else {
                None
            },
            position: (output.rect.x, output.rect.y),
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
            refresh_rate: output.current_mode.map(|mode| mode.refresh),
            workspaces: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputIdentifier(String);

impl From<&SwayOutput> for OutputIdentifier {
    fn from(output: &SwayOutput) -> Self {
        OutputIdentifier(format!(
            "{} {} {}",
            output.make, output.model, output.serial
        ))
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Config(HashMap<OutputIdentifier, OutputProperties>);

impl Config {
    pub fn from_sway_outputs_workspaces(
        outputs: &Vec<SwayOutput>,
        workspaces: &Vec<SwayWorkspace>,
        builtin_outputs: &[OutputIdentifier],
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
            if builtin_outputs.contains(&output_identifier) {
                output_properties.active = None;
            }
            if let Some(workspaces) = output_workspace_map.get(&output.name) {
                if !workspaces.is_empty() {
                    output_properties.workspaces = Some(workspaces.clone());
                }
            }
            new_config.0.insert(output_identifier, output_properties);
        }
        new_config
    }

    pub fn set_in_sway(
        &self,
        outputs: &Vec<SwayOutput>,
        builtin_outputs: &[OutputIdentifier],
        connection: &mut Connection,
    ) {
        let mut commands: Vec<String> = Vec::new();
        for output in outputs {
            let output_identifier = OutputIdentifier::from(output);
            if let Some(output_properties) = self.0.get(&output_identifier) {
                let mut output_properties = output_properties.clone();
                if builtin_outputs.contains(&output_identifier) {
                    output_properties.active = None;
                }
                commands.push(output_properties.to_sway_output_command(&output.name));
                commands.push(output_properties.to_sway_workspace_command(&output.name));
            }
        }

        let cmd_string = commands.clone().join("\n");
        println!("Running commands:\n{}", cmd_string);

        let commands = commands.join(";");
        connection
            .run_command(&commands)
            .expect("Failed to run command");
    }
}

#[derive(Clone, Serialize, Default, Hash, PartialEq, Eq)]
pub struct DefaultConfigIdentifier(Vec<OutputIdentifier>);

impl<'de> Deserialize<'de> for DefaultConfigIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let identifiers: Vec<OutputIdentifier> = Vec::deserialize(deserializer)?;
        let mut sorted_identifiers = identifiers;
        sorted_identifiers.sort();
        Ok(DefaultConfigIdentifier(sorted_identifiers))
    }
}

impl From<&Vec<SwayOutput>> for DefaultConfigIdentifier {
    fn from(outputs: &Vec<SwayOutput>) -> Self {
        let mut identifiers: Vec<OutputIdentifier> =
            outputs.iter().map(OutputIdentifier::from).collect();
        identifiers.sort();
        DefaultConfigIdentifier(identifiers)
    }
}

impl Display for DefaultConfigIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output_names: Vec<String> = Vec::new();
        for output in &self.0 {
            output_names.push(output.0.clone());
        }
        write!(f, "[{}]", output_names.join(", "))
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Hash, PartialEq, Eq)]
pub struct CustomConfigIdentfier(pub String);

impl Display for CustomConfigIdentfier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub builtin_outputs: Vec<OutputIdentifier>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub custom_configurations: HashMap<CustomConfigIdentfier, Config>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub default_configurations: HashMap<DefaultConfigIdentifier, Config>,
}

impl Settings {
    pub fn load_from_file(config_file_path: &PathBuf) -> Settings {
        let config_file =
            std::fs::read_to_string(config_file_path).expect("Unable to read config file");
        serde_yaml::from_str(&config_file).expect("Unable to parse config file")
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
