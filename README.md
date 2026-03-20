<img width="1078" height="592" alt="image" src="https://github.com/user-attachments/assets/4c7c65c7-5afb-4e1d-affb-1d3e02746671" />


*A fast and extensible code editor built on top of [iced](https://iced.rs) that works out of the box.*

Refer to [the website](https://pinel.netlify.app) for further information

---

# Installation

Pinel is shipped across multiple platforms, mainly *Github Releases* and [crates.io](https://crates.io). Note that nightly builds are an ongoing process and are planned to arrive in the near future.

## Quick Install

```bash
curl -fsSL https://pinel.netlify.app/install.sh | sh
```

The installer downloads the latest matching GitHub release for your platform and installs `pinel` into `~/.local/bin` by default. If no prebuilt release asset exists for the current platform, it falls back to `cargo install pinel --locked`.

## Compiling from Source

If you would like to compile from source, please follow the instructions below.

> **Prerequisites** - Ensure you have the `cargo` package manager installed since this project requires Rust and a few of its published crates

1. Clone the repository:

```
git clone https://github.com/kashsuks/pinel
```

2. Change directory
```
cd pinel
```

3. Run the startup command:
```
cargo run
```
You will see that the required packages will get installed and the application may (or may not) run depending on whether the current commit is stable. This will be discussed in the following note.

>[!NOTE]
> The Github commits are not guaranteed to be stable since they are meant to be developer logs (**not user friendly versions**).
> Do not expect them to work because I am a solo developer and I have too much shit to deal with.
> The same applies to nightly builds (when they do arrive).
> They will not be stable. Do not expect them to be stable.

## Mac Installation

- For Mac Silicon (any of the M series chips), please use `pinel-macos-x86_64`

Once you've installed it:
- Copy the path of the install (either through finder or whatever method)
- Go to your terminal of choice and type in `chmod +x <the path to the install>`
- Then run `./pinel-macos-x86_64` while in the correct director

If apple says that the install cannot be verified:
- Go to settings
- Scroll down to `Privacy & Security`
- Scroll all the way down and in the `Security` section it should ask for permission to open
- Press allow and give any other permission
- The app _should_ open!

If you would like to use [crates.io](https://crates.io) to use the application please refer to the [published version](https://crates.io/crates/pinel-editor)

## Arch Linux

For Arch Linux users installation is simple.

On the most recent release, look for `pinel-linux-x86_64` and install it.

Once that is done, run `chmod +x <path of the install>` and then run `./<path of the install>`.
This will make the binary executable and then run it.

## If you cannot access these

Go to [the website](https://pinel.netlify.app) and look at the video demo linked. It will give you can idea of all the features in the project

---

# Features

- Custom theming options (in and out of the editor [GUI](https://en.wikipedia.org/wiki/Graphical_user_interface))
- Fuzzy finding (with Neovim keybinds)
- File tree navigation
- Vim motions (commands) currently for navigation
- Settings/preferences
- System default terminal usage
- Scripting using Lua

--- 

# License

Copyright (c) 2025-Present [Kashyap](https://github.com/kashsuks) and [Contributors](https://github.com/kashsuks/Pinel/graphs/contributors). `Pinel` is a free and open-source software licensed under the [GNU General Public License Version 3](https://www.gnu.org/licenses/gpl-3.0.en.html). Official logo was created by [Kashyap Sukshavasi](https://github.com/kashsuks).
