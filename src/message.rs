use std::path::PathBuf;

use iced_code_editor::LspOverlayMessage;
use iced_term::Event as TerminalEvent;

/// This file is responsible for internal messages
/// Used to send internal flags and data transfer to trigger
/// Other instances of Message types.
use crate::features::search::SearchResult;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    CodeEditorEvent(iced_code_editor::Message),

    LspOverlay(LspOverlayMessage),

    CodeEditorContentChanged,
    FileClicked(PathBuf),
    FileOpened(PathBuf, String),
    SensitiveFileOpenConfirm(bool),
    FolderToggled(PathBuf),

    FileTreeRefresh,
    CursorPositionTracked(f32, f32),
    FileTreeContextMenuOpen(PathBuf, bool),
    FileTreeContextMenuClose,
    FileTreeNewFile,
    FileTreeNewFolder,
    FileTreeRenameStart,
    FileTreeRenameInputChanged(String),
    FileTreeRenameSubmit,
    FileTreeRenameCancel,
    FileTreeReveal,
    FileTreeCopyPath,
    FileTreeDelete,
    /// Fired whenever the active file or open workspace changes, from any
    /// code path. Carries no data — consumers read the new state from
    /// `App::activity_state` when they receive this.
    ActiveFileChanged,
    ToggleSidebar,
    SetActivePanel(crate::app::ActivePanel),

    OpenFolderDialog,
    OpenFileOrFolder,
    FolderOpened(PathBuf),

    SaveFile,
    SaveCurrentFileAs(PathBuf),
    CurrentFileSavedAs(PathBuf),
    FileSaved(Result<(), String>),

    TabSelected(usize),
    TabClosed(usize),
    ReopenClosedTab,
    CloseActiveTab,

    SidebarResizeStart,
    SidebarResizing(f32),
    SidebarResizeEnd,

    PreviewMarkdown,
    MarkdownLinkClicked(iced::widget::markdown::Uri),

    ToggleSearch,
    SearchQueryChanged(String),
    SearchCompleted(Vec<SearchResult>),
    SearchResultClicked(PathBuf, usize),

    ToggleFileFinder,
    FileFinderQueryChanged(String),
    FileFinderSelect,
    FileFinderNavigate(i32),

    ToggleFuzzyFinder,
    FuzzyFinderQueryChanged(String),
    FuzzyFinderSelect,
    FuzzyFinderNavigate(i32),

    ToggleFullscreen(iced::window::Mode),
    EscapePressed,

    ToggleCommandPalette,
    CommandPaletteQueryChanged(String),
    CommandPaletteSelect(String),
    CommandPaletteNavigate(i32),

    TerminalEvent(TerminalEvent),
    ToggleTerminal,
    FocusEditor,
    FocusTerminal,

    ToggleFindReplace,
    FindQueryChanged(String),
    ReplaceQueryChanged(String),
    FindNext,
    FindPrev,
    ReplaceOne,
    ReplaceAll,
    ToggleCaseSensitive,

    ToggleSettings,
    SettingsNavigate(String),
    SettingsTabSizeChanged(String),
    SettingsToggleUseSpaces,
    SettingsToggleAutoIndent,
    SettingsToggleVimMode,
    SettingsToggleDiscordRpc,
    SettingsToggleRestoreSession,
    ToggleLineComment,
    SettingsToggleAutosave,
    SettingsAutosaveIntervalChanged(String),
    SettingsSavePreferences,
    SettingsSelectTheme(String),
    SettingsReloadTheme,
    SettingsLineNumberWidthChanged(String),
    SettingsToggleTabDragFloating,

    ToggleCommandInput,
    CommandInputChanged(String),
    CommandInputSubmit,
    WindowResized(u32, u32),
    WindowCloseRequested(iced::window::Id),

    NewFile,
    SaveAs,
    SelectTabByIndex(usize),
    TabHoverEnter(usize),
    TabHoverExit,
    TabDragInitiate,
    TabHoldTick,
    TabDragMove(f32),
    TabDragEnd,
    TabDragFloatMove(f32, f32),

    WakaTimeApiKeyChanged(String),
    WakaTimeApiKeyHoverStart,
    WakaTimeApiKeyHoverEnd,
    WakaTimeApiUrlChanged(String),
    SaveWakaTimeSettings,

    DismissNotification,
    ModifierStateChanged(iced::keyboard::Modifiers),
    LspTick,
    AutosaveTick,
    DiscordRpcTick,
    SessionSyncTick,
    AutosaveFinished(PathBuf, String, Result<(), String>),

    #[cfg(feature = "unstable-comet")]
    ToggleDeveloperPanel,
    #[cfg(feature = "unstable-comet")]
    ClearDeveloperLogs,
    #[cfg(feature = "unstable-comet")]
    SettingsToggleDeveloperMode,
    ToggleLsp,

    StartupPageDone,
    StartupThemeSelected(String),
    StartupToggleVimMode,
    StartupToggleHelixMode,
    StartupToggleRunOnStartup,

    RefreshGitStatus,
    GitStatusLoaded(Vec<(String, String)>),

    /// Starts a new, empty chat session and makes it the active one.
    ChatNewSession,
    /// Makes the chat session with this id the active one.
    ChatSelectSession(String),
    /// Leaves the active chat session and returns to the history list.
    ChatBackToHistory,
    /// Opens or closes the provider/model picker.
    ChatToggleModelPicker,
    /// Switches which provider's models are shown in the open picker's
    /// right-hand list. Does not commit anything to the session yet.
    ChatPickerProviderSelected(String),
    ChatModelSearchChanged(String),
    /// Commits a (provider, model) pair to the active session and closes
    /// the picker.
    ChatModelPicked(String, String),
    ChatInputChanged(String),
    /// Appends the current input as a user message to the active session
    /// and sends the conversation so far to the session's provider/model.
    ChatSend,
    /// A provider replied (or failed to) to a `ChatSend` for the given
    /// session id.
    ChatResponseReceived(String, Result<String, String>),
    ChatDeleteSession(String),
    ChatRenameStart(String),
    ChatRenameInputChanged(String),
    ChatRenameSubmit,
    ChatRenameCancel,

    /// A model list fetch for a provider (by stable id) finished, either
    /// from the chat picker or the Providers settings panel.
    ProviderModelsFetched(String, Result<Vec<String>, String>),
    /// Switches which provider's credential form is shown in the Providers
    /// settings panel.
    ProvidersSelect(String),
    ProvidersApiKeyChanged(String),
    ProvidersToggleKeyVisibility,
    /// Saves the current form's provider id/key as a credential.
    ProvidersSave,
    /// Clears a saved credential for the given provider id.
    ProvidersRemove(String),
    /// Re-fetches the model list for a provider to confirm its key works.
    ProvidersTestConnection(String),

    CheckForUpdate,
    UpdateAvailable(crate::features::updater::UpdateInfo),
    DismissUpdateBanner,

    // all the ones below are related to media playback
    #[cfg(feature = "video")]
    VideoTogglePause,
    #[cfg(feature = "video")]
    VideoEndOfStream,
    #[cfg(feature = "video")]
    VideoNewFrame,
    AudioPlay,
    AudioPause,
    AudioStop,
    AudioSeek(f32),
    AudioTick,
}
