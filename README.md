# Macro

**Macro** is a powerful, cross-platform CLI tool designed to record and replay mouse and keyboard events. Whether you need to automate repetitive tasks, test UI interactions, or create macros for gaming or productivity, **Macro** provides a simple and efficient solution.

> **Note:** This project is currently under active development. Features and usage patterns are subject to change.

## Features

- **Record Events:** Capture mouse movements, clicks, and keyboard strokes with precision.
- **Replay Events:** Play back recorded sessions with adjustable speed and repeat options.
- **Interactive CLI:** User-friendly terminal interface using `cliclack` for easy navigation.
- **Configurable Hotkeys:** Global hotkeys for starting/stopping recording and playback.
- **Workspace Management:** Organized storage for your recordings and configuration.
- **Cross-Platform:** Works on macOS, Windows, and Linux.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Build from Source

1.  Clone the repository:
    ```bash
    git clone https://github.com/keval8solanki/macro.git
    cd macro
    ```

2.  Build the project:
    ```bash
    cargo build --release
    ```

3.  Run the binary:
    ```bash
    ./target/release/macro
    ```

    Or run directly with Cargo:
    ```bash
    cargo run
    ```

## Usage

**Macro** can be used in an interactive mode or via direct CLI commands.

### Interactive Mode

Simply run the application without arguments to enter the interactive menu:

```bash
macro
```

You will be presented with the following options:

1.  **Config**:
    - Select a workspace folder where your recordings and configuration will be saved.
    - This creates a `config.json` and a `recording/` directory in the selected folder.

2.  **Record**:
    - Enter a name for your recording (defaults to the current timestamp).
    - The tool will wait for the **Start Recording** hotkey.
    - Perform your actions.
    - Press the **Stop Recording** hotkey to save the session.

3.  **Play**:
    - Select a recording from your workspace.
    - Set the **Playback Speed** (e.g., `1.0` for normal, `2.0` for 2x speed).
    - Set the **Repeat Count** (`0` for infinite loop, `1` for single run).
    - The tool will wait for the **Start Playback** hotkey.
    - Press the **Stop Playback** hotkey to interrupt playback at any time.

### CLI Commands

You can also use subcommands for direct execution:

- **Record to a specific file:**
  ```bash
  macro record my_macro.json
  ```

- **Play a specific file:**
  ```bash
  macro play my_macro.json --speed 1.5 --repeat-count 5
  ```

## Hotkeys

Default hotkeys are configured as follows:

| Action | Default Hotkey |
| :--- | :--- |
| **Start Recording** | `Cmd` + `Alt` + `R` |
| **Stop Recording** | `Cmd` + `Alt` + `R` |
| **Start Playback** | `Cmd` + `Alt` + `P` |
| **Stop Playback** | `Cmd` + `Alt` + `P` |

### Customization

You can customize these hotkeys by editing the `config.json` file located in your selected workspace folder.

Example `config.json` snippet:

```json
{
  "keymaps": {
    "start_recording": {
      "modifiers": ["Cmd", "Alt"],
      "trigger": "KeyR"
    },
    ...
  }
}
```
### Allowed Keys

For a complete list of allowed keys, please refer to the [`rdev` documentation](https://docs.rs/rdev/latest/rdev/enum.Key.html).

Common keys include:
- **Letters:** `KeyA` ... `KeyZ`
- **Numbers:** `Num0` ... `Num9`
- **Function Keys:** `F1` ... `F12`
- **Special:** `Return`, `Escape`, `Space`, `Tab`, `Backspace`
- **Modifiers:** `ShiftLeft`, `ControlLeft`, `Alt`, `MetaLeft` (Command/Windows)
