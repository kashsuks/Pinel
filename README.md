<h1 align="center">
  <img width="420" alt="Pinel" src="https://github.com/user-attachments/assets/4c7c65c7-5afb-4e1d-affb-1d3e02746671" />
</h1>

<p align="center">A fast, open-source code editor with no config needed.</p>

<p align="center">
  <a href="https://crates.io/crates/pinel"><img alt="crates.io" src="https://img.shields.io/crates/v/pinel.svg"></a>
  <a href="https://github.com/kashsuks/pinel/actions/workflows/test.yml"><img alt="Tests" src="https://github.com/kashsuks/pinel/actions/workflows/test.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: GPL-3.0" src="https://img.shields.io/badge/license-GPL--3.0-blue.svg"></a>
</p>

## Install

```bash
curl -fsSL https://pinel.netlify.app/install.sh | bash
```

Or with Cargo (Rust 1.88+):

```bash
cargo install pinel --locked
```

Prebuilt binaries are on the [releases page](https://github.com/kashsuks/pinel/releases/latest). There's no Windows build yet.

Run `pinel .` to open the current folder.

## Features

- Vim motions
- Fuzzy finder and command palette
- LSP support (Rust, Python, TS/JS, Go, Lua)
- Git integration
- Built-in terminal
- AI chat (bring your own key)
- Themes and Lua config in `~/.config/pinel/`
- Remote editing over SSH (`pinel connect user@host`)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Questions go in [Discussions](https://github.com/kashsuks/pinel/discussions).

## License

[GPL-3.0](LICENSE)
