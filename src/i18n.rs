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
    let mut mapping = HashMap::<Message, &'static str>::new();

    // store localized messages to mapping here
    let locale = get_current_locale();
    println!("found locale: {}", locale);
    match locale.split_once('-').map(|x| x.0).unwrap_or(locale.as_str()) {
        "ja" => localization_ja(&mut mapping),
        _ => {}
    }

    unsafe {
        LOCALIZED_MAPPING = Some(mapping);
    }
}

fn get_current_locale() -> String {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;
    use windows::Win32::System::SystemServices::LOCALE_NAME_MAX_LENGTH;

    unsafe {
        let mut buffer = [0 as u16; LOCALE_NAME_MAX_LENGTH as usize];
        let len = GetUserDefaultLocaleName(&mut buffer);
        // - 1: remove trailing null char
        String::from_utf16_lossy(&buffer[..(len as usize - 1)])
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
        SelectInGuiButtonText => "Select Folder",

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

        InstallSucceedText => "Installing VRC Log Manager to Task Scheduler succeed!",
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

fn localization_ja(mapping: &mut HashMap<Message, &str>) {
    mapping.insert(ErrorReadingConfigFile, "設定をを読込中にエラーが発生しました");
    mapping.insert(ClickOKToDiscordAndContinue, "OKをクリックすると設定を破棄して続行します");
    mapping.insert(ErrorLoadingConfigFileCaption, "エラー");

    mapping.insert(ErrorWritingConfigFileText, "コンフィグを書き込み中にエラーが発生しました");
    mapping.insert(ErrorWritingConfigFileCaption, "エラー");

    mapping.insert(PathToVrcLogFolder, "VRCのログフォルダのパス");
    mapping.insert(VrcLogFilePattern, "VRCのログファイルのパターン(正規表現)");
    mapping.insert(KeepOriginal, "元ファイルを残す");
    mapping.insert(CopyMoveLogFileTo, "ログファイルの移動先");
    mapping.insert(OutputFilePattern, "ログファイルの出力形式(chronoのstrftime)");
    mapping.insert(UseUcForFileName, "UTCをログファイル名に使用する");
    mapping.insert(SaveConfig, "設定を保存");
    mapping.insert(ResetConfig, "設定を初期化");
    mapping.insert(ExecuteNow, "実行");
    mapping.insert(InstallToTaskScheduler, "Task Schedulerに登録");
    mapping.insert(UninstallFromTaskScheduler, "Task Schedulerの登録解除");
    mapping.insert(SelectInGuiButtonText, "フォルダを選択");

    mapping.insert(SourceFolderChooserCaption, "VRCのログフォルダ");
    mapping.insert(OutputFolderChooserCaption, "出力フォルダ");

    mapping.insert(SaveBeforeCloseText, "閉じる前に保存しますか");
    mapping.insert(SaveBeforeCloseCaption, "閉じる前に保存しますか");

    mapping.insert(ConfigSavedText, "コンフィグが保存されました");
    mapping.insert(ConfigSavedCaption, "コンフィグが保存されました");

    mapping.insert(ResetConfirmText, "本当に初期化しますか");
    mapping.insert(ResetConfirmCaption, "確認");

    mapping.insert(InstallSucceedText, "Task Schedulerへの登録が成功しました");
    mapping.insert(InstallSucceedCaption, "成功");

    mapping.insert(UninstallSucceedText, "Task Schedulerの登録解除が成功しました");
    mapping.insert(UninstallSucceedCaption, "成功");

    mapping.insert(ErrorInRenameText, "実行中にエラーが発生しました");
    mapping.insert(ErrorInRenameCaption, "エラー");

    mapping.insert(RenameSucceedText, "成功しました");
    mapping.insert(RenameSucceedCaption, "成功");

    mapping.insert(InvalidSourcePatternText, "設定の保存に失敗しました: VRCのログファイルのパターンが不正です");
    mapping.insert(InvalidSourcePatternCaption, "エラー");

    mapping.insert(InvalidOutputPatternText, "設定の保存に失敗しました: ログファイルの出力形式が不正です");
    mapping.insert(InvalidOutputPatternCaption, "エラー");
}
