<p align="center">
  <img src="assets/icon.png" width="128" height="128" alt="Macro Logo">
</p>

# Macro

A simple yet powerful macro recorder for macOS, built with Rust.

## Download

You can download the latest version from the [Releases](https://github.com/keval8solanki/macro/releases) page.

> Note:
> **Untrusted Developer**
>
> Since this application is not signed with an Apple Developer ID, macOS may block it from opening.
> 1. Try to open the app.
> 2. If you see a warning, go to **System Settings** -> **Privacy & Security**.
> 3. Scroll down to the security section and look for a message about "Macro.app".
> 4. Click **Open Anyway**.

## Usage

Launch `Macro.app`. The application lives in your system status bar (menu bar).

### Hotkeys
Global hotkeys are available for quick control:

-   **Command + Shift + 1**: Toggle Recording.
    -   **Start**: Begins recording your mouse and keyboard actions.
    -   **Stop**: Stops recording and opens a file dialog to save the macro.
-   **Command + Shift + 2**: Toggle Playback.
    -   **Start**: Plays the currently loaded recording.
    -   **Stop**: Stops the current playback.
-   **Command + Shift + 0**: Load / Unload.
    -   **Load**: Opens a file picker to select a recording (if none loaded).
    -   **Unload**: Unloads the current recording (if one is loaded).

### Status Indicators
The tray icon changes color to indicate the current state:
-   **White**: Idle / Ready.
-   **Red**: Recording in progress.
-   **Orange**: Recording loaded (Armed).
-   **Green**: Playback in progress.

### Settings
Click the tray icon and select **Settings...** to configure:
-   **Playback Speed**: 0.5x, 1.0x, 2.0x, etc.
-   **Repeat Count**: Number of times to loop the macro (or infinite).
-   **Repeat Interval**: Delay between loops.

## Permissions

For the macro recorder to function, it requires specific permissions.

### Accessibility (Required)
Required to record inputs and simulate events.
1.  **System Settings** -> **Privacy & Security** -> **Accessibility**.
2.  Click `+` and add `Macro.app`.
3.  Ensure the toggle is **Enabled**.

### Input Monitoring (Required)
Required to detect and record input events globally.
1.  **System Settings** -> **Privacy & Security** -> **Input Monitoring**.
2.  Add `Macro.app` and enable it.

**Note**: If you update or rebuild the app, you may need to remove and re-add these permissions if macOS invalidates the previous signature.
