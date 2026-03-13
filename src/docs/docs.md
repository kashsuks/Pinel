---
title: Pinel Documentation
description: Learn how to install, configure, and use Pinel.
---

# 1. Introduction

## 1.1 What is Pinel?

Explain at a high level what Pinel is, what problems it solves, and who it is for.

You can cover things like:

- The core idea or philosophy behind Pinel.
- The main features that make it different from other editors.
- Typical workflows or use cases.

## 1.2 Why use it / key concepts

Outline the core concepts a new user should know before diving in.

Examples:

- How projects are organized.
- How configuration works.
- Any important terminology you’ll use throughout the docs.

## 1.3 Architecture (optional)

If useful, briefly describe the high‑level architecture: important components, how they interact, and any constraints or design goals.

# 2. Getting Started

Give a short overview of what’s required to install and run Pinel (supported OS versions, dependencies, etc.).

## 2.1 macOS

Describe how to install and run Pinel on macOS.

You might include:

- System requirements.
- Install via direct download, Homebrew, or source (if applicable).
- First‑run steps and where config files live.

```bash
# Example (replace with real commands)
brew install pinel
pinel
```

## 2.2 Windows

Describe how to install and run Pinel on Windows.

You might include:

- Supported Windows versions.
- Installer vs. portable ZIP.
- Any extra steps (e.g. enabling developer mode, adding to PATH).

```powershell
# Example (replace with real commands)
winget install Pinel
pinel.exe
```

## 2.3 Linux

Describe how to install and run Pinel on Linux distributions.

You might include:

- Generic tarball/ AppImage instructions.
- Distro‑specific notes (Debian/Ubuntu, Fedora, Arch, etc.).

```bash
# Example (replace with real commands)
curl -L https://example.com/pinel.tar.gz -o pinel.tar.gz
tar xzf pinel.tar.gz
./pinel
```

## 2.4 Package managers

Summarize how to install Pinel with different package managers across platforms.

You can group by ecosystem:

- **Homebrew** (macOS / Linux)
- **winget / Chocolatey** (Windows)
- **apt / dnf / pacman** (Linux)

```bash
# Homebrew (macOS / Linux)
brew install pinel

# Debian / Ubuntu
sudo apt install pinel

# Fedora
sudo dnf install pinel

# Arch
sudo pacman -S pinel
```

