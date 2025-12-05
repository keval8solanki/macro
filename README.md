# Macro Recorder

A simple yet powerful macro recorder for macOS, built with Rust.

## Table of Contents
- [Installation](#installation)
- [Build Guide](#build-guide)
- [Usage](#usage)
- [Permissions](#permissions)

## Installation

### Prerequisites
- macOS
- Rust and Cargo (for building from source)

### Steps
1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd macro
    ```

## Build Guide

You can build the application using the included bundle script, which creates a proper macOS `.app` bundle.

1.  Run the bundle script:
    ```bash
    ./bundle.sh
    ```
2.  The application `Macro.app` will be created in the project root.
3.  You can move `Macro.app` to your `/Applications` folder or run it directly.

Alternatively, to just build the binary:
```bash
cargo build --release
```
The binary will be located at `target/release/macro`.

## Usage

Launch `Macro.app` or run the binary. The application runs in the background and lives in your system status bar (menu bar).

### Tray Icon Menu
Click the tray icon to access the menu:
-   **Start/Stop Recording**: Manually toggle recording.
-   **Load Recording**: Open a file picker to select a previously saved JSON recording for playback.
-   **Start/Stop Playback**: Manually toggle playback (requires a loaded recording).
-   **Settings**:
    -   **Playback Speed**: Choose between 0.5x, 1.0x (default), and 2.0x.
    -   **Repeat Count**: Choose between 1x (default) or Infinite loop.
-   **Quit**: Exit the application.

### Hotkeys
Global hotkeys are available for quick control:

-   **Command + Shift + 1**: Toggle Recording.
    -   **Start**: Begins recording your mouse and keyboard actions.
    -   **Stop**: Stops recording and opens a file dialog to save the macro (default location: `Documents/Macros` or workspace configured path).
-   **Command + Shift + 2**: Toggle Playback.
    -   **Start**: Plays the currently loaded recording.
    -   **Stop**: Stops the current playback.

### Status Indicators
The tray icon changes color to indicate status:
-   **White**: Idle / Ready.
-   **Red**: Recording in progress.
-   **Yellow**: Recording loaded / Playback in progress.

## Permissions

For the macro recorder to function (record inputs and simulate events), it requires **Accessibility** permissions on macOS.

### How to Grant Permissions
1.  **First Run**: When you first run the app and try to record or play, macOS may prompt you to grant Accessibility access. Click "Open System Settings".
2.  **Manual Setup**:
    -   Open **System Settings**.
    -   Go to **Privacy & Security** -> **Accessibility**.
    -   Click the `+` button in the list.
    -   Select `Macro.app` (or your Terminal app if running from CLI).
    -   Ensure the toggle is enabled.

**Note**: If you rebuild the app, you might need to remove and re-add the entry in Accessibility settings if macOS doesn't recognize the new binary signature.

### Troubleshooting
-   If usage is "jerky" or events are dropped, ensure **Input Monitoring** permission is also granted (though Accessibility is usually sufficient for `rdev` and `tao`).
-   If hotkeys are not working, ensure another app isn't blocking them or that the app has Accessibility permissions.
