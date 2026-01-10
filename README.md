# niri-app-hotkey

A command-line tool for managing application windows in the [Niri](https://github.com/YaLTeR/niri) Wayland compositor. This tool allows you to launch, show, hide, activate, and toggle application windows using hotkey bindings or script integration.

## ⚠️ Important Requirement

**This tool requires [Niri PR #2997](https://github.com/YaLTeR/niri/pull/2997) to function properly.** This PR introduces the workspace hiding functionality that this tool depends on to implement window hiding operations. Without this PR, the `hide`, `toggle`, and other window manipulation commands will not work as expected.

Additionally, **you must configure at least one hidden workspace** in your Niri configuration for the window hiding feature to work. A hidden workspace is used to store hidden windows.

Example Niri configuration:

```kdl
workspace "stash" {
    hidden true
}
```

Make sure your Niri installation includes the changes from PR #2997 and that you have configured a hidden workspace before using niri-app-hotkey.

### Niri Installation with PR #2997

If you're using Arch Linux, you can directly install Niri with PR #2997 using the AUR package from [niri-git](https://github.com/GoodbyeNJN/niri-git):

```bash
git clone https://github.com/GoodbyeNJN/niri-git.git
cd niri-git
makepkg -si
```

This AUR package includes the necessary changes from PR #2997, ensuring compatibility with niri-app-hotkey.

## Features

- **Launch applications** - Start applications using configured spawn commands
- **Window management** - Show, hide, activate, or intelligently toggle application windows
- **Flexible matching** - Match windows by application ID and window title (compatible with Niri's window rules)
- **Window selection** - Select specific windows by index when multiple windows match
- **Exclusion rules** - Exclude specific windows from matching criteria
- **Configuration-based** - Easy configuration using KDL configuration language
- **Default configuration path** - Automatically reads from `$XDG_CONFIG_HOME/niri/niri-app-hotkey.kdl`

## Installation

### Build from source

```bash
cargo build --release
```

The compiled binary will be available at `target/release/niri-app-hotkey`.

## Usage

### Command Syntax

```bash
niri-app-hotkey [OPTIONS] <COMMAND>
```

### Options

- `-c, --config <PATH>` - Path to configuration file (defaults to `$XDG_CONFIG_HOME/niri/niri-app-hotkey.kdl`)
- `-h, --help` - Print help message
- `-V, --version` - Print version information

### Commands

#### `validate`

Validates the configuration file syntax without performing any actions.

```bash
niri-app-hotkey validate
```

#### `launch <APP_NAME>`

Launches the specified application using its configured command.

```bash
niri-app-hotkey launch "Telegram"
```

#### `show <APP_NAME>`

Shows the window(s) of the specified application that match the configured rules.

```bash
niri-app-hotkey show "Firefox"
```

#### `hide <APP_NAME>`

Hides the window(s) of the specified application that match the configured rules.

```bash
niri-app-hotkey hide "Firefox"
```

#### `activate <APP_NAME>`

Activates (brings to focus) the window(s) of the specified application that match the configured rules.

```bash
niri-app-hotkey activate "Firefox"
```

#### `toggle <APP_NAME>`

Intelligently toggles the specified application with the following behavior:

1. **No matching windows** - Launches the application using the configured spawn command
2. **Hidden matching window** - Shows the window
3. **Visible but inactive window** - Activates (brings into focus) the window
4. **Active window** - Hides the window

This command is ideal for binding to hotkeys, providing a single-key control for toggling application visibility.

```bash
niri-app-hotkey toggle "Telegram"
```

## Configuration

The configuration file uses the KDL (KDL Document Language) format. By default, it's located at:

```
$XDG_CONFIG_HOME/niri/niri-app-hotkey.kdl
```

### Configuration Format

Each application configuration block contains:

- **name** - The unique identifier for the application (used in commands)
- **spawn** or **spawn-sh** - Command to launch the application (at least one is required; use `spawn` for direct execution or `spawn-sh` for shell command execution)
- **match** - Rules to identify windows belonging to this application
- **exclude** - Rules to exclude specific windows from matching

### Spawn Command

The `spawn` directive specifies the command to execute when launching the application. It accepts a list of arguments where the first element is the command name and subsequent elements are arguments. This is the recommended method for launching applications directly without shell interpretation.

The behavior follows the same logic as [Niri's spawn action](https://yalter.github.io/niri/Configuration%3A-Key-Bindings.html#spawn): the command is executed with the specified arguments, with support for path expansion (e.g., `~` for home directory).

Examples:

```kdl
// Simple command
spawn "firefox"

// Command with arguments
spawn "gtk-launch" "org.telegram.desktop.desktop"

// Command with path
spawn "code" "/path/to/project"
```

### Spawn Shell Command

The `spawn-sh` directive executes a shell command string directly using `sh -c`. This is useful for commands with complex shell features like pipes, redirects, or variable expansion.

The behavior follows the same logic as [Niri's spawn-sh action](https://yalter.github.io/niri/Configuration%3A-Key-Bindings.html#spawn-sh).

Examples:

```kdl
// Simple shell command
spawn-sh "firefox &"

// Command with pipe
spawn-sh "echo 'Starting app' | notify-send"

// Command with environment variables
spawn-sh "DISPLAY=:1 some-app"
```

### Match and Exclude Rules

The `match` directives identify which windows should be targeted by the application. The `exclude` directives explicitly exclude windows from matching. The matching behavior for `app-id` and `title` follows the same logic as [Niri's window rules](https://yalter.github.io/niri/Configuration%3A-Window-Rules.html):

- A window must match **any** of the `match` directives (OR logic)
- A window must **not** match **any** of the `exclude` directives

Supported matchers:

| Property | Type   | Description                                                           | Notes                |
| -------- | ------ | --------------------------------------------------------------------- | -------------------- |
| `app-id` | Regex  | Match windows by application ID                                       | Same as Niri         |
| `title`  | Regex  | Match windows by window title                                         | Same as Niri         |
| `index`  | Number | Select the N-th window from the matched candidates (0-based indexing) | niri-app-hotkey only |

Both `app-id` and `title` support regular expressions. You can find the app-id and title of a window using:

```bash
niri msg pick-window
```

Then click on the window you want to match.

#### Window Selection with Index

When multiple windows match your `match` and `exclude` rules, the `index` property allows you to select a specific window instead of operating on all matching windows:

1. The tool first filters all windows using the `match` and `exclude` rules, creating a candidate window list
2. The candidate list is then sorted by process ID (PID), with lower PIDs appearing first
3. If an `index` is specified, only the window at that position in the sorted list is operated on
4. If no `index` is specified, all matching windows are operated on

This is useful when an application has multiple windows and you want to target a specific one:

```kdl
// Operate on all matching Firefox windows
application "Firefox-All" {
    spawn "firefox"
    match app-id="firefox"
}

// Operate only on the first Firefox window (by PID)
application "Firefox-Primary" {
    spawn "firefox"
    match app-id="firefox" index=0
}

// Operate only on the second Firefox window (by PID)
application "Firefox-Secondary" {
    spawn "firefox"
    match app-id="firefox" index=1
}
```

## Configuration Examples

### Example 1: Simple Application

```kdl
application "Firefox" {
    spawn "gtk-launch" "firefox"
    match app-id="firefox"
}
```

### Example 2: Multiple Match Rules

```kdl
application "IDE" {
    spawn "code"
    match app-id="code" title="Visual Studio Code"
}
```

### Example 3: Window Indexing

```kdl
application "Terminal-2" {
    spawn "gnome-terminal"
    match app-id="org.gnome.Terminal" index=1
}
```

### Example 4: With Exclusions

```kdl
application "Chat" {
    spawn "telegram-desktop"
    match app-id="org\.telegram\.desktop"
    exclude title="Notification"
}
```

## Configuration File Locations

**Note:** This tool only supports Linux systems, as Niri itself is only available for Linux.

The configuration file is searched in the following order:

1. **Explicit path** - Via `-c` or `--config` option
2. **Default path** - `$XDG_CONFIG_HOME/niri/niri-app-hotkey.kdl`
    - Typically `~/.config/niri/niri-app-hotkey.kdl` on most Linux distributions

## Integration with Niri Configuration

To use `niri-app-hotkey` with Niri hotkeys, add key bindings to your Niri configuration:

```kdl
binds {
    // Toggle Telegram with Super+T
    Super+T { spawn "niri-app-hotkey" "toggle" "Telegram"; }

    // Show Firefox with Super+F
    Super+F { spawn "niri-app-hotkey" "show" "Firefox"; }

    // Hide current window with Super+H
    Super+H { spawn "niri-app-hotkey" "hide" "Firefox"; }
}
```

## Building and Development

### Prerequisites

- Rust 1.90 or later
- Cargo

### Building

```bash
cargo build
```

### Running

```bash
cargo run -- -c example_config.kdl validate
```

### Testing Configuration

```bash
niri-app-hotkey -c ~/.config/niri/niri-app-hotkey.kdl validate
```

## Troubleshooting

### Configuration Validation

To validate your configuration file syntax:

```bash
niri-app-hotkey validate -c ~/.config/niri/niri-app-hotkey.kdl
```

For detailed error messages, the tool will display parse errors with context.

### Configuration-Related Issues

For issues related to `match` and `exclude` rules, window detection, or KDL syntax, refer to:

- [Niri Window Rules documentation](https://yalter.github.io/niri/Configuration%3A-Window-Rules.html) - For understanding window matching
- [Rust regex documentation](https://docs.rs/regex/latest/regex/#syntax) - For regular expression syntax

### Other Issues

If you encounter issues not covered by the above, please open an issue on the project's GitHub repository with detailed information about your problem.

## Acknowledgments

We would like to thank:

- [Niri](https://github.com/YaLTeR/niri) - An amazing Wayland window manager that makes this tool possible
- [Niri PR #2997](https://github.com/YaLTeR/niri/pull/2997) - For introducing the workspace hiding functionality that powers window hiding in this tool
- [niri-scratchpad-rs](https://github.com/argosnothing/niri-scratchpad-rs/tree/hidden-workspaces) - An awesome utility that served as inspiration for this project

## License

See the project's license file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
