# aks-terminal

A GPU-accelerated terminal emulator written in Rust.

- GPU rendering via [wgpu](https://wgpu.rs)
- Frameless window with a built-in title bar (minimize / maximize / close, drag to move)
- Scrollback with mouse-wheel scrolling
- Themeable colors (16-color ANSI palette)
- Automatic bash shell integration (installed on first run, no config needed)

## Install (Debian / Ubuntu)

Download the `.deb` and install it with `dpkg`:

```bash
wget https://github.com/Akash-Kumar-Sinha/terminal-emulator/releases/download/v0.1.0/aks-terminal_0.1.0-1_amd64.deb
```

```bash
sudo dpkg -i aks-terminal_0.1.0-1_amd64.deb
```

You can also download it manually from the
[Releases](https://github.com/Akash-Kumar-Sinha/terminal-emulator/releases) page.

Launch it from your application menu, or run `aks-terminal` in a terminal.

## Requirements

- Linux (x86_64)
- A GPU with a working Vulkan/OpenGL driver
- `bash` (for shell integration)
