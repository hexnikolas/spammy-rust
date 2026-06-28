# Spammy - Rust X11 Fork

![Spammy app screenshot]([https://example.com/your-screenshot.png](https://i.imgur.com/9G1KBMi.png)

This repository is a Rust fork of the original Spammy project at https://github.com/FrostAtom/spammy.
It is written in Rust and built for X11 on Linux, with a native `egui` UI.

## What this is

- Fork of https://github.com/FrostAtom/spammy
- Native Rust implementation using `egui`
- Designed for X11 on Linux
- Input device selection via `/dev/input/event*`
- Synthetic key sending through Linux input

## Key behavior

- **Spammy keys** (orange)
  - Left-click a key in the UI to mark it as Spammy
  - When the app is enabled and the physical key is held down, the key repeats automatically
  - Useful for held actions in games or apps where repeated press behavior is desired

- **Speedy keys** (yellow)
  - Right-click a key in the UI to mark it as Speedy
  - When the key is pressed, the app sends a single fast tap
  - Useful for quick one-shot actions instead of holding the key down

- **Normal keys** (gray)
  - No special automation
  - Behavior remains the same as the physical key press

## Features

- Rust-based Linux/X11 key repeater
- Visual keyboard layout with key state indicators
- Profile support for saving key sets and device settings
- Input device selection for manual keyboard path
- Efficient timer-driven repeat implementation

## Architecture

- `main.rs` - App entry point and eframe initialization
- `app.rs` - Application state, profile logic, and key mode handling
- `keyboard.rs` - Keyboard layout definitions and key rendering
- `input_handler.rs` - Linux evdev input detection
- `key_sender.rs` - Synthetic input sending to the system
- `profile.rs` - Profile and settings persistence
- `ui.rs` - egui-based user interface

## Requirements

### System Dependencies

```bash
# Ubuntu/Debian
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel

# Arch
sudo pacman -S libxcb libxkbcommon openssl
```

### Permissions

This app requires elevated permissions to:
1. Read keyboard input from `/dev/input/eventX`
2. Send synthetic keys via `/dev/uinput`

## Building

```bash
cd spammy-rust
cargo build --release
```

## Running

```bash
sudo ./target/release/spammy
```

## Desktop menu / start menu

To make Spammy appear in the Linux application menu for other users:

1. Install the binary to a standard path, for example:
   - `/usr/local/bin/spammy`
   - or `~/.local/bin/spammy`
2. Create a `.desktop` file at `~/.local/share/applications/spammy.desktop` or `/usr/share/applications/spammy.desktop`
3. Optionally add an icon at `~/.local/share/icons/hicolor/256x256/apps/spammy.png`

Example `spammy.desktop`:

```ini
[Desktop Entry]
Name=Spammy
Comment=Rust X11 key repeater
Exec=sudo /usr/local/bin/spammy
Icon=spammy
Terminal=false
Type=Application
Categories=Utility;Game;
```

After this, Spammy should appear in the start menu under Utilities or Games.

## Usage

1. Start the app with `sudo`
2. Select the input device if needed
3. Use the on-screen keyboard to toggle key modes:
   - left-click = Spammy (orange)
   - right-click = Speedy (yellow)
4. Enable the app and press the physical key to trigger the selected behavior

## Configuration

Repeat interval and profile settings are managed in the profile system.
Profiles store active keys, speedy keys, repeat timing, and input device selection.

## Notes

This project is intended for X11 on Linux and is a fork of the original Spammy utility.

## Troubleshooting

### "No keyboard device found"

The app couldn't locate your keyboard in `/dev/input/`. Check which device is your keyboard:

```bash
cat /proc/bus/input/devices | grep -A5 "keyboard\|Keyboard"
# Look for the event number, e.g., "event6"
```

Then modify `input_handler.rs` to use the correct device.

### "Permission denied" on `/dev/uinput`

Ensure you're running with `sudo`. Alternatively, add your user to the `input` group:

```bash
sudo usermod -a -G input $USER
sudo usermod -a -G uinput $USER
# Log out and back in
```


## Performance

The low-lag design follows these principles from Linux input best practices:

- **inotify/epoll**: Event-driven, not polling
- **Timer-based execution**: Only send keys every X ms
- **Direct uinput**: No subprocess overhead (vs xdotool)
- **Main loop sleep**: 10ms to let OS breathe



## Building for Distribution

```bash
# Static build
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release

# Debug symbols for debugging
cargo build --release --keep-going
```

