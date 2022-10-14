use std::collections::HashMap;
use Message::*;

#[derive(Eq, PartialEq, Hash)]
pub enum Message {
    ErrorReadingConfigFile,
    ClickOKToDiscordAndContinue,
    ErrorLoadingConfigFileCaption,

    ErrorWritingConfigFileText,
    ErrorWritingConfigFileCaption,

    PathToVrcLogFolder,
    VrcLogFilePattern,
    KeepOriginal,
    CopyMoveLogFileTo,
    OutputFilePattern,
    UseUcForFileName,
    SaveConfig,
    ResetConfig,
    ExecuteNow,
    InstallToTaskScheduler,
    UninstallFromTaskScheduler,
    SelectInGuiButtonText,

    SourceFolderChooserCaption,
    OutputFolderChooserCaption,

    SaveBeforeCloseText,
    SaveBeforeCloseCaption,

    ConfigSavedText,
    ConfigSavedCaption,

    ResetConfirmText,
    ResetConfirmCaption,

    InstallSucceedText,
    InstallSucceedCaption,

    UninstallSucceedText,
    UninstallSucceedCaption,

    ErrorInRenameText,
    ErrorInRenameCaption,

    RenameSucceedText,
    RenameSucceedCaption,

    InvalidSourcePatternText,
    InvalidSourcePatternCaption,

    InvalidOutputPatternText,
    InvalidOutputPatternCaption,
}

macro_rules! m {
    ($name: expr) => {
        $crate::i18n::get_message($name)
    };
}

static mut LOCALIZED_MAPPING: Option<HashMap<Message, &'static str>> = None;

pub fn init_i18n() {
    let mapping = HashMap::<Message, &'static str>::new();

    // store localized messages to mapping here

    unsafe {
        LOCALIZED_MAPPING = Some(mapping);
    }
}

pub fn get_message(message: Message) -> &'static str {
    unsafe {
        let mapping = LOCALIZED_MAPPING.as_ref().expect("i18n not initialized");
        if let Some(msg) = mapping.get(&message) {
            return msg;
        }
    }
    // fallback to english
    match message {
        ErrorReadingConfigFile => "Error reading config file",
        ClickOKToDiscordAndContinue => "Click OK to discord config & continue.",
        ErrorLoadingConfigFileCaption => "Error",

        ErrorWritingConfigFileText => "Error writing config file",
        ErrorWritingConfigFileCaption => "Error",

        PathToVrcLogFolder => "Path to VRC Log Folder:",
        VrcLogFilePattern => "VRC Log File Pattern (regex):",
        KeepOriginal => "Keep Original",
        CopyMoveLogFileTo => "Copy/Move Log file to:",
        OutputFilePattern => "Output File Pattern (chrono's strftime):",
        UseUcForFileName => "Use UTC Time for log name",
        SaveConfig => "Save Config",
        ResetConfig => "Reset Config",
        ExecuteNow => "Execute Now",
        InstallToTaskScheduler => "Install to Task Scheduler",
        UninstallFromTaskScheduler => "Uninstall from Task Scheduler",
        SelectInGuiButtonText => "Select",

        SourceFolderChooserCaption => "VRC Log Folder",
        OutputFolderChooserCaption => "Output Folder",

        SaveBeforeCloseText => "Save Config before Close?",
        SaveBeforeCloseCaption => "Save?",

        ConfigSavedText => "Config Saved!",
        ConfigSavedCaption => "Config Saved!",

        ResetConfirmText => {
            "Are you sure want to reset config to default?\nYou cannot undo this operation"
        }
        ResetConfirmCaption => "Confirm?",

        InstallSucceedText => "Installing VRC Log Manager from Task Scheduler succeed!",
        InstallSucceedCaption => "Succeed!",

        UninstallSucceedText => "Uninstalling VRC Log Manager from Task Scheduler succeed!",
        UninstallSucceedCaption => "Succeed!",

        ErrorInRenameText => "Error during renaming logs",
        ErrorInRenameCaption => "Error!",

        RenameSucceedText => "Renaming Log Succeed!",
        RenameSucceedCaption => "Succeed!",

        InvalidSourcePatternText => "Cannot save the config: Log file Pattern is not valid",
        InvalidSourcePatternCaption => "Error",

        InvalidOutputPatternText => "Cannot save the config: Output File Pattern is not valid",
        InvalidOutputPatternCaption => "Error",
    }
}
