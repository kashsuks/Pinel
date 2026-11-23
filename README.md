<h1 align="center">
  <img width="420" alt="Pinel" src="https://github.com/user-attachments/assets/4c7c65c7-5afb-4e1d-affb-1d3e02746671" />
</h1>

<p align="center">
  <strong>A fast, open-source code editor that works out of the box.</strong><br>
  Vim motions, fuzzy finding, language servers, git and a terminal are built in, so there's no config to write before you start.
</p>

<p align="center">
  <a href="https://crates.io/crates/pinel"><img alt="crates.io" src="https://img.shields.io/crates/v/pinel.svg"></a>
  <a href="https://github.com/kashsuks/pinel/actions/workflows/test.yml"><img alt="Tests" src="https://github.com/kashsuks/pinel/actions/workflows/test.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/license-GPL--3.0-blue.svg"></a>
</p>

<p align="center">
  <a href="https://pinel.netlify.app/?utm_source=github&utm_medium=readme&utm_campaign=pinel">Watch the demo</a> ·
  <a href="https://github.com/kashsuks/pinel/discussions">Ask a question</a> ·
  <a href="https://github.com/kashsuks/pinel/issues/new">Report a bug</a>
</p>

<!-- TODO: add a screenshot or short GIF (under 5 MB) of the editor itself: open a folder, fuzzy-find a file, hover a symbol -->


## Quickstart

```bash
curl -fsSL https://pinel.netlify.app/install.sh | bash
pinel .
```

The first command installs the latest release for macOS or Linux (x86_64 or arm64) into `~/.local/bin`. The second opens the current folder in Pinel. If you get `pinel: command not found`, see [Troubleshooting](#troubleshooting).

## Features

- **Vim motions** for moving around a file. Turn them on per tab with <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>V</kbd>, or for every new tab in Settings.
- **Fuzzy finder and command palette** to open any file or run any command from the keyboard.
- **Language servers** for hover docs and go-to-definition in Rust, Python, TypeScript/JavaScript, Go and Lua. Pinel finds servers you installed with rustup, npm, pip, Go or Homebrew.
- **Git** status, diffs, commits and a commit graph, without leaving the editor.
- **Integrated terminal** running your default shell.
- **AI chat** with your own API key for OpenAI, Anthropic, OpenRouter, Hack Club AI or Kimi. Chat history is saved per project.
- **Themes and Lua scripting**, set from inside the editor or from files in `~/.config/pinel/`.
- **Remote browsing over SSH** from the terminal with `pinel connect user@host`.
- **Markdown preview**, plus Discord Rich Presence and WakaTime integrations.

### Keyboard shortcuts

<kbd>Cmd</kbd> on macOS, <kbd>Ctrl</kbd> on Linux.

| Action | Shortcut |
|---|---|
| Go to file | <kbd>Cmd</kbd>+<kbd>T</kbd> |
| Fuzzy file search with preview | <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> |
| Command palette | <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd> |
| Find and replace | <kbd>Cmd</kbd>+<kbd>F</kbd> |
| Toggle terminal | <kbd>Cmd</kbd>+<kbd>J</kbd> |
| Toggle sidebar | <kbd>Cmd</kbd>+<kbd>B</kbd> |
| Go to definition | <kbd>F12</kbd> |
| Markdown preview | <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> |
| Settings | <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>S</kbd> |

## Install

The [quickstart](#quickstart) script covers macOS and Linux. Prebuilt Windows releases aren't available yet.

### From crates.io

```bash
cargo install pinel --locked
```

This needs Rust 1.88 or newer. Run `rustup update stable` if your toolchain is older.

### Download a release by hand

1. Download the zip for your machine from the [latest release](https://github.com/kashsuks/pinel/releases/latest):

   | Machine | File |
   |---|---|
   | Mac with Apple Silicon (M1 or later) | `pinel-macos-arm64.zip` |
   | Mac with an Intel chip | `pinel-macos-x86_64.zip` |
   | Linux, x86_64 | `pinel-linux-x86_64.zip` |
   | Linux, arm64 | `pinel-linux-arm64.zip` |

2. Unzip it and move the `pinel` binary onto your `PATH`:

   ```bash
   unzip pinel-macos-arm64.zip
   chmod +x pinel
   mv pinel ~/.local/bin/
   ```

### Build from source

```bash
git clone https://github.com/kashsuks/pinel
cd pinel
git checkout v0.9.0
cargo install --path . --locked
```

> [!NOTE]
> `master` is where day-to-day development happens, and any commit on it may be broken. For a stable build, install a tagged release: the quickstart script and crates.io both do. Nightly builds, once they exist, won't be stable either.

## Configuration

Pinel works without any config. To change things, open Settings (<kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>S</kbd>) or edit the files in `~/.config/pinel/`:

| File | What it controls |
|---|---|
| `preferences.lua` | Editor preferences |
| `theme.lua` | Colors. See [color customization](DOCS/CODE_EDITOR_COLOR_CUSTOMIZATION.md). |
| `init.lua` | Your own Lua scripts, run at startup |

In **Settings → Preferences → Text and display**, adjust UI zoom (80–150%), editor and terminal text sizes (12–24 logical pixels), and Regular/Medium code weight. Changes preview immediately; click **Save Preferences** to keep them. UI zoom is applied on top of your operating system's display scaling. Explorer/Git and chat panel widths are remembered separately when you finish resizing them.

To restyle the hover and autocomplete popups, see [LSP overlay customization](DOCS/LSP_OVERLAY_CUSTOMIZATION.md).

## Troubleshooting

**`pinel: command not found`**

`~/.local/bin` isn't on your `PATH`. Add this line to `~/.zshrc` or `~/.bashrc`, then open a new terminal:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**macOS says Pinel "cannot be verified"**

This happens when you download the zip in a browser. Either run:

```bash
xattr -d com.apple.quarantine ~/.local/bin/pinel
```

or open **System Settings → Privacy & Security**, scroll to **Security**, and click **Open Anyway** next to the message about Pinel.

**Hover and go-to-definition don't work**

Pinel uses the language server for each language, so it has to be installed:

| Language | Install |
|---|---|
| Rust | `rustup component add rust-analyzer` |
| Python | `npm install -g pyright` |
| TypeScript / JavaScript | `npm install -g typescript-language-server typescript` |
| Go | `go install golang.org/x/tools/gopls@latest` |
| Lua | `brew install lua-language-server` |

## Contributing

Bug reports and pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers running the tests, enabling the git hooks and the `utils/clean.sh` quality checks. For questions and ideas, use [Discussions](https://github.com/kashsuks/pinel/discussions).

## License

Copyright © 2025–present [Kashyap](https://github.com/kashsuks) and [contributors](https://github.com/kashsuks/pinel/graphs/contributors). Pinel is free, open-source software licensed under the [GNU General Public License v3.0](LICENSE). The official logo was created by [Kashyap Sukshavasi](https://github.com/kashsuks).
