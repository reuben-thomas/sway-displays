# sway-displays
### Installation
``` Bash
cargo install --path .
```
### Usage

``` 
Usage: sway-displays [OPTIONS] <COMMAND>

Commands:
  list            List all saved configurations
  show-connected  Show names of connected displays
  save            Save current as a default configuration
  save-custom     Save current layout as a custom configuration
  set             Automatically set a default configuration based on connected displays
  set-custom      Set a custom configuration by name
  run             Run in continuous mode and automatically set apply default configurations based on connected displays
  help            Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG_FILE_PATH>  Use a custom config file
  -h, --help                       Print help
```
### Configuration
Unless otherwise specified through the `-c` flag, the default config file should be placed as `~/.config/sway-displays/config.yml` 
``` yaml
custom_configurations:
  # Cusom configurations are identified by name
  Home Setup:
    ...
  Work Setup:
    ...
default_configurations:
  # Default configurations are identified by a set of connected displays
  ? [
      Dell Inc. DELL S3422DWG 48LXSH3,
      "Goldstar Company Ltd LG FULL HD ",
      Sharp Corporation 0x1515 0x00000000,
    ]
  : "Goldstar Company Ltd LG FULL HD ":
      active: true
      resolution: [1920, 1080]
      position: [3440, 230]
      rotation: "90"
      refresh_rate: 60000
      scale: 0.95
      workspaces: [music, 1, 2, 3, 4]
    Dell Inc. DELL S3422DWG 48LXSH3:
      ...
    Sharp Corporation 0x1515 0x00000000:
      ...
```
