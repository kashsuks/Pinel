use std::path::PathBuf;

/// This file is responsible for starting up the process
/// and defining as well as starting up the cli tool
/// for ssh connection
use clap::Parser;
use iced::window;

mod app;
mod autocomplete;
mod cli;
mod config;
mod discord_rpc;
mod features;
mod fs_util;
mod lsp_setup;
mod message;
mod scripting;
mod ssh;
mod subscriptions;
mod theme;
mod tui;
mod ui;
mod wakatime;

const FIRA_CODE_BOLD: &[u8] = include_bytes!("assets/fonts/FiraCode-Bold.ttf");
const FIRA_CODE_REGULAR: &[u8] = include_bytes!("assets/fonts/FiraCode-Regular.ttf");
const FIRA_CODE_MEDIUM: &[u8] = include_bytes!("assets/fonts/FiraCode-Medium.ttf");
const SF_PRO: &[u8] = include_bytes!("assets/fonts/SF-Pro.ttf");

fn main() -> iced::Result {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(cli::Commands::Connect { target, path }) => {
            if let Err(error) = tui::remote_browser::run(target, path) {
                eprintln!("connect failed: {error:#}");
                std::process::exit(1);
            }
            Ok(())
        },
        None => run_gui(cli.path),
    }
}

fn run_gui(startup_path: Option<PathBuf>) -> iced::Result {
    // `pinel .` passes a relative path, and language servers only attach to
    // absolute ones. If the working directory is unreadable, keep it as given.
    let startup_path = startup_path.map(|path| match std::env::current_dir() {
        Ok(cwd) => fs_util::absolute_path(&path, &cwd),
        Err(_) => path,
    });

    lsp_setup::ensure_lsp_paths();
    iced_code_editor::ensure_rust_analyzer_config();

    let icon_data = include_bytes!("assets/icon.png"); // when an icon is created, set it to src/assets/icon.png
    let icon = window::icon::from_file_data(icon_data, None).expect("Failed to load icon.");
    let prefs = config::preferences::load_preferences();
    let window_width = prefs.window_width.max(640.0);
    let window_height = prefs.window_height.max(480.0);

    iced::application(
        move || app::App::new_with_path(startup_path.clone()),
        app::App::update,
        app::App::view,
    )
    .title("Pinel")
    .scale_factor(app::App::ui_scale)
    .subscription(|app| app.subscription())
    .font(FIRA_CODE_BOLD)
    .font(FIRA_CODE_REGULAR)
    .font(FIRA_CODE_MEDIUM)
    .font(SF_PRO)
    .default_font(iced::Font {
        family: iced::font::Family::Name("SF Pro"),
        weight: iced::font::Weight::Medium,
        ..iced::Font::DEFAULT
    })
    .window_size((window_width, window_height))
    .window(window::Settings {
        size: [window_width, window_height].into(),
        icon: Some(icon),
        exit_on_close_request: false,
        ..Default::default()
    })
    .run()
}
