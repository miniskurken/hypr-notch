# hypr-notch

**hypr-notch** is a modular, extensible notch overlay for Wayland compositors, written in Rust. It is designed to provide a customizable "notch" area at the top of your screen, similar to the macOS notch, but with support for user modules and dynamic content.

## Features

- **Wayland-native:** Integrates directly with Wayland compositors using [smithay-client-toolkit].
- **Modular System:** Easily extend functionality by writing your own modules (e.g., clock, status, custom widgets).
- **Dynamic Rendering:** Smooth drawing with support for transparency, rounded corners, and custom backgrounds.
- **Configurable:** Reads configuration from a TOML file (`~/.config/hypr-notch/config.toml`), allowing you to adjust size, colors, and enabled modules.
- **Pointer and Input Handling:** Supports pointer events for interactive modules.
- **Layer Shell Protocol:** Uses the wlr-layer-shell protocol to position the notch overlay above other windows.

## Architecture Overview

- **Entry Point (`main.rs`):** Initializes configuration, connects to the Wayland server, sets up the event loop, and ties together all components.
- **App State (`app.rs`):** Manages application state, surface configuration, drawing, and module updates.
- **Drawing (`draw.rs`):** Provides utilities for rendering, including a simple canvas abstraction and text rendering.
- **Modules (`modules/`, `module/`):** Contains built-in modules (like the clock) and the module interface/registry system for extensibility.
- **Wayland Integration (`wayland.rs`):** Handles Wayland protocol events, surface configuration, and input events.

## Getting Started

1. **Build:**  
   ```sh
   cargo build --release
   ```

2. **Run:**  
   ```sh
   cargo run --release
   ```

3. **Configure:**  
   Edit `~/.config/hypr-notch/config.toml` to customize appearance and enabled modules.

## Example Configuration

```toml
collapsed_width = 300
collapsed_height = 40
expanded_width = 800
expanded_height = 400
corner_radius = 20
background_color = [0, 0, 0, 255]

[modules]
enabled = ["clock"]
[module_configs.clock]
color = [255, 255, 255, 255]
format = "%H:%M:%S"
font_size = 16.0
```

## Writing Your Own Module

Implement the `Module` trait (see `src/module/interface.rs`) and register your module in the registry. Modules can handle events, draw on the canvas, and define their own configuration.

## License

MIT

---

*hypr-notch is not affiliated with Hyprland or Apple. It is an independent open-source project.*
