# niri-app-hotkey

A command-line tool for managing application windows in the [Niri](https://github.com/YaLTeR/niri) Wayland compositor. This tool allows you to launch, show, hide, focus, and toggle application windows using hotkey bindings or script integration.

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
- **Window management** - Show, hide, focus, or intelligently toggle application windows
- **Flexible matching** - Match windows by application ID and window title (compatible with Niri's window rules)
- **Window selection** - Select a specific window by index and configurable sort order when multiple windows match
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

Moves the matching window to the currently focused workspace if necessary, then focuses it.

```bash
niri-app-hotkey show "Firefox"
```

#### `hide <APP_NAME>`

Moves the matching focused window to the hidden workspace. The matching window must already be focused.

```bash
niri-app-hotkey hide "Firefox"
```

#### `focus <APP_NAME>`

Focuses the matching window if it is on the currently focused workspace. Unlike `show`, this command does not move a window between workspaces.

```bash
niri-app-hotkey focus "Firefox"
```

#### `toggle <APP_NAME>`

Intelligently toggles the specified application with the following behavior:

1. **No matching window** - Launches the application using its configured spawn command
2. **Matching window is focused** - Hides it by moving it to the hidden workspace
3. **Matching window is not focused** - Moves it to the currently focused workspace if necessary, then focuses it

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
- **spawn** or **spawn-sh** - Optional command to launch the application (required if an action needs to launch it)
- **window** - Optional window-matching rules, including `match`, `exclude`, and `select`

### Spawn Command

The `spawn` directive specifies the command to execute when launching the application. It accepts a list of arguments where the first element is the command name and subsequent elements are arguments. This is the recommended method for launching applications directly without shell interpretation.

The command is executed with the specified arguments. A leading `~` in the executable path is expanded to the home directory.

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

Window rules are declared inside a `window` block. The `match` directives identify candidate windows, and `exclude` directives remove candidates from consideration. The matching behavior for `app-id` and `title` follows the same logic as [Niri's window rules](https://yalter.github.io/niri/Configuration%3A-Window-Rules.html):

- A window must match **at least one** `match` directive (OR logic across directives)
- Within one directive, all specified properties must match (AND logic)
- A window is excluded if it matches **any** `exclude` directive

Supported matchers:

| Property | Type  | Description                    | Notes        |
| -------- | ----- | ------------------------------ | ------------ |
| `app-id` | Regex | Match windows by application ID | Same as Niri |
| `title`  | Regex | Match windows by window title   | Same as Niri |

Both `app-id` and `title` support regular expressions. You can find the app-id and title of a window using:

```bash
niri msg pick-window
```

Then click on the window you want to match.

#### Selecting a Window

Each action operates on one matching window. If more than one window matches, specify a `select` block to choose a target with a zero-based `index`:

1. The tool filters windows using the `match` and `exclude` rules.
2. It sorts the candidates by the keys listed in `order-by`, in the listed priority order.
3. It selects the window at the specified zero-based `index`.

If there is only one matching window, it is selected automatically. If multiple windows match and no `select` is configured, the command reports an error. Supported `order-by` keys are `id`, `title`, `app-id`, `pid`, and `workspace-id`. With no `order-by`, candidates use ascending Niri window ID order.

```kdl
application "Firefox-Primary" {
    spawn "firefox"
    window {
        match app-id="firefox"
        select index=0 {
            order-by pid
        }
    }
}

// Operate on the second Firefox window (by PID)
application "Firefox-Secondary" {
    spawn "firefox"
    window {
        match app-id="firefox"
        select index=1 {
            order-by pid
        }
    }
}
```

## Configuration Examples

### Example 1: Simple Application

```kdl
application "Firefox" {
    spawn "gtk-launch" "firefox"
    window {
        match app-id="firefox"
    }
}
```

### Example 2: Multiple Match Rules

```kdl
application "IDE" {
    spawn "code"
    window {
        match app-id="code" title="Visual Studio Code"
    }
}
```

### Example 3: Window Indexing

```kdl
application "Terminal-2" {
    spawn "gnome-terminal"
    window {
        match app-id="org.gnome.Terminal"
        select index=1 {
            order-by pid
        }
    }
}
```

### Example 4: With Exclusions

```kdl
application "Chat" {
    spawn "telegram-desktop"
    window {
        match app-id="org\.telegram\.desktop"
        exclude title="Notification"
    }
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

    // Show (and focus) Firefox with Super+F
    Super+F { spawn "niri-app-hotkey" "show" "Firefox"; }

    // Hide Firefox if its matching window is focused
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
