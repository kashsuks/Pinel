---
title: Pinel Documentation
description: Learn how to install, configure, and use Pinel.
---

# 1. Introduction

## 1.1 What is Pinel?

Pinel is a code-editor written using the Rust programming language as well as the popular [Iced Library](https://iced.rs/) for GUI rendering.
Pinel aims to bridge some gaps between Visual Studio Code as well as editors like Neovim and Helix by being really fast and extremely customizable.

## 1.2 Why use it / key concepts

If you've ever found:
- VSCode (Visual Studio Code) too slow or too much of a gimmick
- Neovim/Vim a bit too complicated (too high of a learning curve)

And if you wanted an editor that just gets things done without throwing everything at you, Pinel might be a good choice. It is in no means a fully fledged editor that I recommend you switch over to, but it is in a decently okay state that you could use for testing and such.

Here are some of the features that are currently working in Pinel:
- Integrated terminal support
    - If you use MacOS, your `zsh` terminal works perfectly fine in Pinel using the `iced-term` package for Rust
    - Not tested for Windows but will be tested and fixed in the next `@stable` release for Pinel. (keep an eye out on Crates and Github Releases)
- LSP (Language Server Protocol) support
    - Out-of-the-box LSP support for Rust (rust-analzyer), Python (), Go, C/C++, and more!
    - Supports Autocomplete, and Hover Documentation
    - De-loads LSP for inactive files (especially useful for rust-analyzer due to memory intensity)
- Custom theming
    - By using Lua you are able to theme your code-editor to your liking
    - Soon to have internal API that can be called using Lua for scripting
- Wakatime support
    - Enjoy tracking time? Or part of Hack Club?
    - Set your own Wakatime URL and API key to send heartbeats to any server of your choice
    - Uses wakatime-cli

## 1.3 Architecture

Pinel uses [Iced](https://iced.rs) for the GUI rendering due to its support for ligatures (allows uses to use nerd fonts), [iced-code-editor](https://github.com/LuDog71FR/iced-code-editor) for core features such as LSP support and line numbers, [iced-term](https://github.com/Harzu/iced_term) for integrated terminal support as well as many other packages and their functions mentioned under the [CREDITS.md](https://github.com/kashsuks/Pinel/blob/master/CREDITS.md) file for Pinel. Go check them out!

# 2. Getting Started

Pinel currently has active support for the following:
- Mac Silicon (M1, M2, M3, etc)
- Arch Linux
- [Rust Package Manager Crates.io](https://crates.io)
<!-- - [Arch Linux User Repository](https://aur.archlinux.org/) -->

There is planned support in the future for the following:
- Other Linux distros
- Homebrew (MacOS)
- Winget/Chocolatey

## 2.1 macOS

Lucky for you, I develop on MacOS therefore every version (Stable, Dev, Alpha, etc) is guaranteed to have every feature on MacOS. There have been plans to have the ability to use a .DMG file for users to easily install however that requires `Gatekeeper` access for Mac meaning I would need a developer account. 

That aside, here is how you can install of MacOS.

You can use the following command and paste it in your terminal:
```bash
curl -fsSL https://pinel.netlify.app/install.sh | sh
```
And the script will install the editor for usage!

You can also install via the `pinel-macos-x86_64` under the latest release which can be found [here](https://github.com/kashsuks/Pinel/releases)

Due to the extra security that MacOS has for unverified apps, you will need to grant the app some permissions (that Gatekeeper would otherwise give by default). Here are the steps to follow:
- Copy the path of your installation
- Go to your terminal of choice and type in `chmod +x <the path to the install>`
- Then run `./pinel-macos-x86_64`

*Optional*: You can also add this new binary to your `$PATH` if you would like to just refer to the app as `Pinel`

**For users looking for a Homebrew installation, hang tight it will be available soon**

## 2.2 Windows

There is no windows installation currently due to cross-platform dependency issues. A solution will be found soon

**There is no active option for Winget or Chocolatey however they are in the works.**

## 2.3 Linux

The easiest way to install on Linux is with the install script:
```bash
curl -fsSL https://pinel.netlify.app/install.sh | sh
```
The script automatically detects your architecture and downloads the correct binary.

Alternatively, grab the binary directly from the [latest release](https://github.com/kashsuks/Pinel/releases):

- **x86_64 (most desktops/servers):** `pinel-linux-x86_64`
- **ARM64 (Raspberry Pi, ARM servers):** `pinel-linux-arm64`

Once downloaded, run `chmod +x <path of the install>` and then `./<path of the install>` to make it executable and launch it.

## 2.4 Package managers

Pinel currently has support through the following package managers:

- [Crates.io](https://crates.io/crates/pinel)
```bash
cargo install pinel
```

## 3. Internal API

### 3.1 Supported Clients

The only supported client as of now would be using `lua` to invoke the API functions. Support for other languages such as `python` or `javascript` are plans for the future.

*Note*: All configurations must be done through `pinel/init.lua` and the `pinel` folder being located in the dotfiles of your device

### pinel.theme.use_builtin(name)

This function allows you to use one of the builtin themes supported by pinel. They are:
- Nord
- TokyoNight
- ...

Example:
```lua
pinel.theme.use_builtin("Nord")
```
Having this line of code in the `init.lua` file will make sure that the theme is *Nord* by default

### pinel.theme.set_color(name, hex)

Allows you to override a named UI or editor colour slot with hex colour.

Params:
- `name`: Name of the ui component
- `hex`: Hex value for the override

Example:
```lua
pinel.theme.set_color("bg_status_bar", "#101722")
pinel.theme.set_color("editor.current_line_highlight", "#7aa2f733")
```

All of the options for `name` are listed [here](https://github.com/kashsuks/Pinel/blob/master/examples/init.lua)

### pinel.ui.show_sidebar(visible)

Boolean that enables or disables the sidebar on startup

Example:
```lua
pinel.ui.show_sidebar(false)
```
It can also be set to `true`

### pinel.ui.set_sidebar_width(width)

Sets the sidebar width in pixels on startup. Width is clamped to the editors allowed range.

**Allowed range is 20px - 120px**

Example
```lua
pinel.ui.set_sidebar_width(100.0)
```

*TODO*: Not allow the user to exceed these bounds/have a reminder that they cannot

### Extra Resources

A full example file is located [here](https://github.com/kashsuks/Pinel/blob/master/examples/init.lua) that you may refer to.

## 4 Basic Commands

In there future, the scripting API written for Lua will support custom keybinds and macros as well as a UI system for bindings.

### 4.1 Opening

You can either open a file or a whole folder. These are the commands.

- `Command/Control + O`: Opening a specific file
- `Command/Control + Shift + O`: Opening a folder

The command palette is useful since it allows you to control almost everything by just typing it in. Here is how to access it.

- `Command/Control + Shift + P`: Opens the command palette