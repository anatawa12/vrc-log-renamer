use std::cell::UnsafeCell;
#[cfg(target_env = "gnu")]
use winsafe_qemu as winsafe;

use crate::config::{parse_pattern, read_config, save_config, ConfigFile, Output, Source};
use crate::i18n::init_i18n;
use crate::i18n::Message::*;
use crate::task_managers::{register_task_manager, unregister_task_manager};
use crate::{config_file_path, rename_main};
use anyhow::{bail, Result};
use regex::Regex;
use winsafe::co::FOS;
use winsafe::co::{DLGID, MB};
use winsafe::prelude::*;
use winsafe::prelude::{user_Hwnd, GuiNativeControlEvents, GuiWindow};
use winsafe::SHCreateItemFromParsingName;
use winsafe::{co, CoCreateInstance, IFileOpenDialog};
use winsafe::{gui, HWND, POINT, SIZE};

pub fn gui_main() -> Result<()> {
    init_i18n();

    let config = read_config_with_error_dialog()?;

    println!("config loaded.");

    let gui = MainGUI::new();
    gui.lazy_load_config(config);
    gui.run()?;

    Ok(())
}

fn read_config_with_error_dialog() -> Result<ConfigFile> {
    match read_config() {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("error reading config: {:?}", e);
            let message = format!(
                "{}: {}.\n{}",
                m!(ErrorReadingConfigFile),
                e,
                m!(ClickOKToDiscordAndContinue)
            );
            if HWND::GetDesktopWindow().MessageBox(
                &message,
                m!(ErrorLoadingConfigFileCaption),
                MB::OKCANCEL,
            )? == DLGID::OK
            {
                eprintln!("error ignored, continue with default config");
                Ok(Default::default())
            } else {
                bail!(e)
            }
        }
    }
}

fn save_config_with_error_dialog(config: &ConfigFile) -> Result<()> {
    match save_config(config) {
        Ok(()) => println!("config file written to: {}", config_file_path().display()),
        Err(e) => {
            eprintln!("error writing config: {:?}", e);
            let message = format!("{}: {}.", m!(ErrorWritingConfigFileText), e);
            HWND::GetDesktopWindow().MessageBox(
                &message,
                m!(ErrorWritingConfigFileCaption),
                MB::OK,
            )?;
            bail!(e);
        }
    }
    Ok(())
}

struct MainGUI {
    window: gui::WindowMain,
    inputs: GUIInputs,
    save_config: gui::Button,
    reset_to_default: gui::Button,
    run_renamer: gui::Button,
    install: gui::Button,
    uninstall: gui::Button,
}

#[derive(Clone)]
struct GUIInputs {
    source_folder: FileSelectBlock,
    source_pattern: TextInputBlock,
    source_keep_original: gui::CheckBox,
    output_folder: FileSelectBlock,
    output_pattern: TextInputBlock,
    output_use_utc: gui::CheckBox,
}

const TEXT_HEIGHT: i32 = 18;

#[inline(always)]
fn check_state(checked: bool) -> gui::CheckState {
    if checked {
        gui::CheckState::Checked
    } else {
        gui::CheckState::Unchecked
    }
}

impl MainGUI {
    pub fn new() -> Self {
        let window = gui::WindowMain::new(
            // instantiate the window manager
            gui::WindowMainOpts {
                title: "VRC Log Renamer".to_owned(),
                size: SIZE::new(400, 323),
                ..Default::default() // leave all other options as default
            },
        );

        let mut y_pos = 10;
        let space = 7;

        let source_folder = FileSelectBlock::new(
            &window,
            m!(PathToVrcLogFolder).to_owned(),
            String::new(),
            POINT::new(10, y_pos),
            380,
        );
        y_pos += FileSelectBlock::HEIGHT + space;

        let source_pattern = TextInputBlock::new(
            &window,
            m!(VrcLogFilePattern).to_owned(),
            String::new(),
            POINT::new(10, y_pos),
            380,
        );
        y_pos += TextInputBlock::HEIGHT + space;

        let source_keep_original = gui::CheckBox::new(
            &window,
            gui::CheckBoxOpts {
                text: m!(KeepOriginal).to_owned(),
                check_state: gui::CheckState::Indeterminate,
                position: POINT::new(10, y_pos),
                ..Default::default()
            },
        );
        y_pos += TEXT_HEIGHT + space * 2;

        let output_folder = FileSelectBlock::new(
            &window,
            m!(CopyMoveLogFileTo).to_owned(),
            String::new(),
            POINT::new(10, y_pos),
            380,
        );
        y_pos += FileSelectBlock::HEIGHT + space;

        let output_pattern = TextInputBlock::new(
            &window,
            m!(OutputFilePattern).to_owned(),
            String::new(),
            POINT::new(10, y_pos),
            380,
        );
        y_pos += TextInputBlock::HEIGHT + space;

        let output_use_utc = gui::CheckBox::new(
            &window,
            gui::CheckBoxOpts {
                text: m!(UseUcForFileName).to_owned(),
                check_state: gui::CheckState::Indeterminate,
                position: POINT::new(10, y_pos),
                ..Default::default()
            },
        );
        y_pos += TEXT_HEIGHT + space;

        let save_config = gui::Button::new(
            &window,
            gui::ButtonOpts {
                text: m!(SaveConfig).to_owned(),
                position: POINT::new(10, y_pos),
                width: 120,
                height: 23,
                ..Default::default()
            },
        );

        let reset_to_default = gui::Button::new(
            &window,
            gui::ButtonOpts {
                text: m!(ResetConfig).to_string(),
                position: POINT::new(140, y_pos),
                width: 120,
                height: 23,
                ..Default::default()
            },
        );

        let run_renamer = gui::Button::new(
            &window,
            gui::ButtonOpts {
                text: m!(ExecuteNow).to_owned(),
                position: POINT::new(270, y_pos),
                width: 120,
                height: 23,
                ..Default::default()
            },
        );

        y_pos += 23 + space;

        let install = gui::Button::new(
            &window,
            gui::ButtonOpts {
                text: m!(InstallToTaskScheduler).to_owned(),
                position: POINT::new(10, y_pos),
                width: 185,
                height: 23,
                ..Default::default()
            },
        );

        let uninstall = gui::Button::new(
            &window,
            gui::ButtonOpts {
                text: m!(UninstallFromTaskScheduler).to_owned(),
                position: POINT::new(205, y_pos),
                width: 185,
                height: 23,
                ..Default::default()
            },
        );

        let new_self = Self {
            window,
            inputs: GUIInputs {
                source_folder,
                source_pattern,
                source_keep_original,
                output_folder,
                output_pattern,
                output_use_utc,
            },
            save_config,
            reset_to_default,
            run_renamer,
            install,
            uninstall,
        };
        new_self.events(); // attach our events
        new_self
    }

    pub fn load_values_from_config(&self, config: &ConfigFile) {
        self.inputs.load_values_from_config(config);
    }

    pub fn run(&self) -> gui::MsgResult<i32> {
        self.window.run_main(None) // simply let the window manager do the hard work
    }

    fn lazy_load_config(&self, config: ConfigFile) {
        self.window.on().wm_activate({
            let optional = UnsafeCell::new(Some((config, self.inputs.clone())));
            move |_| {
                if let Some((config, inputs)) = unsafe { (*optional.get()).take() } {
                    inputs.load_values_from_config(&config);
                }
                Ok(())
            }
        });
    }

    fn events(&self) {
        self.inputs.events(&self.window);
        self.window.on().wm_close({
            let window = self.window.clone();
            let inputs = self.inputs.clone();
            move || {
                if window.hwnd().MessageBox(
                    m!(SaveBeforeCloseText),
                    m!(SaveBeforeCloseCaption),
                    MB::YESNO,
                )? == DLGID::YES
                {
                    inputs.create_save_config(window.hwnd())?;
                }
                window.hwnd().DestroyWindow()?;
                Ok(())
            }
        });
        self.save_config.on().bn_clicked({
            let window = self.window.clone();
            let inputs = self.inputs.clone();
            move || {
                if let Some(Some(_)) = inputs.create_save_config(window.hwnd()).ok() {
                    window.hwnd().MessageBox(
                        m!(ConfigSavedText),
                        m!(ConfigSavedCaption),
                        MB::OK,
                    )?;
                }
                Ok(())
            }
        });
        self.reset_to_default.on().bn_clicked({
            let window = self.window.clone();
            let inputs = self.inputs.clone();
            move || {
                if window.hwnd().MessageBox(
                    m!(ResetConfirmText),
                    m!(ResetConfirmCaption),
                    MB::OKCANCEL,
                )? == DLGID::OK
                {
                    inputs.load_values_from_config(&Default::default());
                }
                Ok(())
            }
        });
        self.install.on().bn_clicked({
            let window = self.window.clone();
            let inputs = self.inputs.clone();
            move || {
                if let Some(Some(_)) = inputs.create_save_config(window.hwnd()).ok() {
                    register_task_manager()?;
                    window.hwnd().MessageBox(
                        m!(InstallSucceedText),
                        m!(InstallSucceedCaption),
                        MB::OK,
                    )?;
                }
                Ok(())
            }
        });
        self.uninstall.on().bn_clicked({
            let window = self.window.clone();
            let inputs = self.inputs.clone();
            move || {
                if let Some(Some(_)) = inputs.create_save_config(window.hwnd()).ok() {
                    unregister_task_manager()?;
                    window.hwnd().MessageBox(
                        m!(UninstallSucceedText),
                        m!(UninstallSucceedCaption),
                        MB::OK,
                    )?;
                }
                Ok(())
            }
        });
        self.run_renamer.on().bn_clicked({
            let window = self.window.clone();
            let inputs = self.inputs.clone();
            move || {
                if let Some(Some(new_config)) = inputs.create_save_config(window.hwnd()).ok() {
                    if let Some(e) = rename_main(&new_config).err() {
                        eprintln!("error during rename: {:?}", e);
                        window.hwnd().MessageBox(
                            &format!("{}: {}", m!(ErrorInRenameText), e),
                            m!(ErrorInRenameCaption),
                            MB::OK,
                        )?;
                    } else {
                        window.hwnd().MessageBox(
                            m!(RenameSucceedText),
                            m!(RenameSucceedCaption),
                            MB::OK,
                        )?;
                    }
                }
                Ok(())
            }
        });
    }
}

impl GUIInputs {
    pub(crate) fn events(&self, window: &(impl GuiParent + Clone + 'static)) {
        self.source_folder
            .events(window, m!(SourceFolderChooserCaption));
        self.source_pattern.events();
        self.output_folder
            .events(window, m!(OutputFolderChooserCaption));
        self.output_pattern.events();
    }

    pub fn load_values_from_config(&self, config: &ConfigFile) {
        self.source_folder
            .set_text(config.source().folder().to_string_lossy().as_ref());
        self.source_pattern
            .set_text(config.source().pattern().as_str());
        self.source_keep_original
            .set_check_state(check_state(config.source().keep_old()));
        self.output_folder
            .set_text(config.output().folder().to_string_lossy().as_ref());
        self.output_pattern
            .set_text(config.output().pattern_as_string().as_str());
        self.output_use_utc
            .set_check_state(check_state(config.output().utc_time()));
    }

    pub fn create_config(&self, window: HWND) -> Result<Option<ConfigFile>, co::ERROR> {
        let source_pattern = match Regex::new(&self.source_pattern.text()) {
            Ok(pat) => pat,
            Err(_) => {
                window.MessageBox(
                    m!(InvalidSourcePatternText),
                    m!(InvalidSourcePatternCaption),
                    MB::OK,
                )?;
                return Ok(None);
            }
        };
        let output_pattern = match parse_pattern(&self.output_pattern.text()) {
            Some(pat) => pat,
            None => {
                window.MessageBox(
                    m!(InvalidOutputPatternText),
                    m!(InvalidOutputPatternCaption),
                    MB::OK,
                )?;
                return Ok(None);
            }
        };
        Ok(Some(ConfigFile::new(
            Source::new(
                self.source_folder.text().into(),
                source_pattern,
                self.source_keep_original.is_checked(),
            ),
            Output::new(
                self.output_folder.text().into(),
                output_pattern,
                self.output_use_utc.is_checked(),
            ),
        )))
    }

    pub(crate) fn create_save_config(&self, hwnd: HWND) -> Result<Option<ConfigFile>, co::ERROR> {
        if let Some(new_config) = self.create_config(hwnd)? {
            if let Some(_) = save_config_with_error_dialog(&new_config).ok() {
                return Ok(Some(new_config));
            }
        }
        Ok(None)
    }
}

fn add_point(a: POINT, b: POINT) -> POINT {
    POINT::new(a.x + b.x, a.y + b.y)
}

#[derive(Clone)]
struct FileSelectBlock {
    _label: gui::Label,
    edit: gui::Edit,
    select: gui::Button,
}

impl FileSelectBlock {
    const HEIGHT: i32 = 41;

    fn new(
        window: &impl GuiParent,
        name: String,
        initial: String,
        origin: POINT,
        width: u32,
    ) -> FileSelectBlock {
        Self {
            _label: gui::Label::new(
                window,
                gui::LabelOpts {
                    text: name,
                    position: add_point(origin, POINT::new(0, 0)),
                    ..Default::default()
                },
            ),
            edit: gui::Edit::new(
                window,
                gui::EditOpts {
                    text: initial,
                    position: add_point(origin, POINT::new(0, TEXT_HEIGHT)),
                    width: width - 100,
                    height: 23,
                    ..Default::default()
                },
            ),
            select: gui::Button::new(
                window,
                gui::ButtonOpts {
                    text: m!(SelectInGuiButtonText).to_owned(),
                    position: add_point(origin, POINT::new((width - 90) as i32, TEXT_HEIGHT)),
                    width: 90,
                    height: 23,
                    ..Default::default()
                },
            ),
        }
    }

    fn text(&self) -> String {
        self.edit.text()
    }

    pub(crate) fn set_text(&self, text: &str) {
        self.edit.set_text(text)
    }

    pub(crate) fn events(&self, window: &(impl GuiParent + Clone + 'static), title: &'static str) {
        self.select.on().bn_clicked({
            let window = window.clone();
            let edit = self.edit.clone();
            move || {
                let obj = CoCreateInstance::<IFileOpenDialog>(
                    &co::CLSID::FileOpenDialog,
                    None,
                    co::CLSCTX::INPROC_SERVER,
                )?;
                obj.SetTitle(&title)?;
                if let Some(item) = SHCreateItemFromParsingName(&edit.text(), None).ok() {
                    obj.SetFolder(&item)?;
                }
                obj.SetFileName(&edit.text())?;
                obj.SetOptions(FOS::PICKFOLDERS)?;
                if obj.Show(window.hwnd())? {
                    let path = obj.GetResult()?.GetDisplayName(co::SIGDN::FILESYSPATH)?;
                    edit.set_text(&path);
                    println!("folder chosen: {}", path);
                }
                Ok(())
            }
        });
    }
}

#[derive(Clone)]
struct TextInputBlock {
    _label: gui::Label,
    edit: gui::Edit,
}

impl TextInputBlock {
    const HEIGHT: i32 = 41;

    fn new(
        window: &impl GuiParent,
        name: String,
        initial: String,
        origin: POINT,
        width: u32,
    ) -> Self {
        Self {
            _label: gui::Label::new(
                window,
                gui::LabelOpts {
                    text: name,
                    position: add_point(origin, POINT::new(0, 0)),
                    ..Default::default()
                },
            ),
            edit: gui::Edit::new(
                window,
                gui::EditOpts {
                    text: initial,
                    position: add_point(origin, POINT::new(0, TEXT_HEIGHT)),
                    width,
                    height: 23,
                    ..Default::default()
                },
            ),
        }
    }

    fn text(&self) -> String {
        self.edit.text()
    }

    pub(crate) fn set_text(&self, text: &str) {
        self.edit.set_text(text)
    }

    pub(crate) fn events(&self) {}
}
