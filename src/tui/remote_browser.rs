use crate::ssh::session::{RemoteEntry, SshSession};
use anyhow::{bail, Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

pub fn run(target: String, start_path: String) -> Result<()> {
    let session = SshSession::connect(&target)?;
    let current_dir = session.canonicalize(&start_path)?;
    let mut browser = RemoteBrowser::new(target, session, current_dir)?;
    browser.run()
}

struct RemoteBrowser {
    target: String,
    session: SshSession,
    current_dir: PathBuf,
    entries: Vec<RemoteEntry>,
    selected: usize,
    status: String,
}

impl RemoteBrowser {
    fn new(target: String, session: SshSession, current_dir: PathBuf) -> Result<Self> {
        let entries = session.read_dir(&current_dir)?;
        Ok(Self {
            target,
            session,
            current_dir,
            entries,
            selected: 0,
            status: String::from("Up/Down: move Enter: open Backspace: up q: quit"),
        })
    }

    fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Up => self.previous(),
                    KeyCode::Down => self.next(),
                    KeyCode::Backspace => self.go_up()?,
                    KeyCode::Enter => self.activate()?,
                    _ => {}
                }
            }
        };

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        result
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(frame.area());
        let main = layout[0];
        let footer = layout[1];

        let items = self.entries.iter().map(|entry| {
            let prefix = if entry.is_dir { "[D]" } else { "[F]" };
            ListItem::new(format!("{prefix} {}", entry.name))
        });

        let mut state = ListState::default();
        state.select((!self.entries.is_empty()).then_some(self.selected));

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("{}: {}", self.target, self.current_dir.display()))
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));

        frame.render_stateful_widget(list, main, &mut state);
        frame.render_widget(Paragraph::new(self.status.clone()), footer);
    }

    fn next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1) % self.entries.len();
        }
    }

    fn previous(&mut self) {
        if !self.entries.is_empty() {
            self.selected = if self.selected == 0 {
                self.entries.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.entries = self.session.read_dir(&self.current_dir)?;
            self.selected = 0;
        }
        Ok(())
    }

    fn activate(&mut self) -> Result<()> {
        let Some(entry) = self.entries.get(self.selected).cloned() else {
            return Ok(());
        };

        if entry.is_dir {
            self.current_dir = entry.path;
            self.entries = self.session.read_dir(&self.current_dir)?;
            self.selected = 0;
            return Ok(());
        }

        self.edit_remote_file(&entry.path)
    }

    fn edit_remote_file(&mut self, remote_path: &Path) -> Result<()> {
        let temp_dir = tempdir().context("failed to create temp dir for remote file")?;
        let file_name = remote_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let local_path = temp_dir.path().join(file_name);

        self.session.download_file(remote_path, &local_path)?;
        self.status = format!("editing {}", remote_path.display());

        let current_exe =
            std::env::current_exe().context("failed to resolve current executable")?;
        let editor_status = Command::new(current_exe)
            .arg(&local_path)
            .status()
            .context("failed to launch Pinel editor")?;

        if !editor_status.success() {
            bail!("Pinel editor exited unsuccessfully");
        }

        self.session.upload_file(&local_path, remote_path)?;
        self.status = format!("saved {}", remote_path.display());
        Ok(())
    }
}
