// VRC Log Renamer - the tool to rename logs of VRChat to have date info
// Copyright (C) 2022 anatawa12
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

mod config;

#[cfg(target_env = "gnu")]
use winsafe_qemu as winsafe;

use crate::config::{read_config, save_config, ConfigFile};
use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use once_cell::race::OnceBox;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};
use take_if::TakeIf;
use winsafe::co::{DLGID, KF, KNOWNFOLDERID, MB};
use winsafe::prelude::{user_Hwnd, GuiNativeControlEvents, GuiWindow};
use winsafe::{gui, SHGetKnownFolderPath, HWND, POINT, SIZE};

fn main() -> Result<()> {
    let config = match read_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("error reading config: {:?}", e);
            let message = format!(
                "Error reading config file: {}.\nClick OK to discord config & continue.",
                e
            );
            if HWND::GetDesktopWindow().MessageBox(&message, "Error", MB::OKCANCEL)? == DLGID::OK {
                eprintln!("error ignored, continue with default config");
                Default::default()
            } else {
                bail!(e)
            }
        }
    };

    println!("config loaded.");

    MainGUI::new().run()?;
    //rename_main(&config)?;

    match save_config(&config) {
        Ok(()) => println!("config file written to: {}", config_file_path().display()),
        Err(e) => {
            eprintln!("error writing config: {:?}", e);
            let message = format!("Error writing config file: {}.", e);
            HWND::GetDesktopWindow().MessageBox(&message, "Error", MB::OK)?;
            bail!(e);
        }
    };

    Ok(())
}

struct MainGUI {
    wnd: gui::WindowMain,   // responsible for managing the window
    btn_hello: gui::Button, // a button
}

impl MainGUI {
    pub fn new() -> Self {
        let wnd = gui::WindowMain::new(
            // instantiate the window manager
            gui::WindowMainOpts {
                title: "My window title".to_owned(),
                class_icon: gui::Icon::Id(101), // load icon from resource ID 101
                size: SIZE::new(300, 150),
                ..Default::default() // leave all other options as default
            },
        );

        let btn_hello = gui::Button::new(
            &wnd, // the window manager is the parent of our button
            gui::ButtonOpts {
                text: "&Click me".to_owned(),
                position: POINT::new(20, 20),
                ..Default::default()
            },
        );

        let new_self = Self { wnd, btn_hello };
        new_self.events(); // attach our events
        new_self
    }

    pub fn run(&self) -> gui::MsgResult<i32> {
        self.wnd.run_main(None) // simply let the window manager do the hard work
    }

    fn events(&self) {
        self.btn_hello.on().bn_clicked({
            let self2 = self.wnd.clone(); // clone so it can be passed into the closure
            move || {
                self2.hwnd().SetWindowText("Hello, world!")?;
                Ok(())
            }
        });
    }
}

fn rename_main(config: &ConfigFile) -> Result<()> {
    let out_folder = config.output().folder();
    fs::create_dir_all(out_folder)?;
    for entry in fs::read_dir(config.source().folder())? {
        let entry = entry?;
        if config
            .source()
            .pattern()
            .is_match(&entry.file_name().to_string_lossy())
        {
            if let Some(err) = move_log_file(config, &entry.path()).err() {
                eprintln!("error moving '{}': {}", entry.path().display(), err);
            }
        }
    }
    Ok(())
}

fn move_log_file(config: &ConfigFile, path: &Path) -> io::Result<()> {
    // first, try to open as read to check if the log file is not of running VRChat
    let mut file = match fs::File::options().write(true).read(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            println!("{} may be used by other process. skipping", path.display());
            return Ok(());
        }
    };
    // then, assume launch time
    let date = assume_launch_time(&mut file)?;
    // now, close the file.
    drop(file);

    // Data to copy log is ready. Now, move/copy log file.

    fs::create_dir_all(config.output().folder())?;
    let dst_path = config.output().folder().join(format!(
        "{}",
        date.format_with_items(config.output().pattern().iter())
    ));

    if dst_path.exists() {
        // if there's file at dst, we assume copy/move is done
        println!(
            "{} exists. we assume output log is already copied",
            dst_path.display()
        );
        return Ok(());
    }

    if config.source().keep_old() {
        // copy log file
        fs::copy(path, dst_path)?;
    } else {
        // move log file
        move_file(path, dst_path)?;
    }

    Ok(())
}

fn assume_launch_time(f: &mut fs::File) -> io::Result<DateTime<Utc>> {
    // length of "%Y.%m.%d %H:%M:%S" is 19 bytes
    let mut buffer = [0 as u8; 19];
    f.read_exact(&mut buffer)?;
    // it must be ascii.
    let str = std::str::from_utf8(&buffer)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf8"))?;
    let time_from_log = chrono::NaiveDateTime::parse_from_str(str, "%Y.%m.%d %H:%M:%S")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid VRC log"))?;

    /*
    // TODO: creation time based time zone inference
    let creation_time = match f.metadata()?.created() {
        Ok(time) => Some(time),
        Err(ref e) if e.kind() == io::ErrorKind::Unsupported => None,
        Err(e) => return Err(e),
    };
    let creation_time = creation_time.map(DateTime::<Utc>::from);
    if let Some(creation_time) = creation_time {
        // if there's creation time and the minute & second is close to time_from_log,
        // use time difference between two for time zone inference

    }
     */

    Ok(time_from_log.and_local_timezone(Utc).earliest().unwrap())
}

#[cfg(windows)]
// ERROR_NOT_SAME_DEVICE
static CROSSES_DEVICES_OS_CODE: i32 = 17;

fn move_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fn move_by_copy(from: &Path, to: &Path) -> io::Result<()> {
        let mut from_file = fs::File::options().read(true).write(true).open(from)?;
        let mut to_file = fs::File::options().create_new(true).write(true).open(to)?;
        io::copy(&mut from_file, &mut to_file)?;
        to_file.flush()?;
        drop(from_file);
        drop(to_file);
        fs::remove_file(from)?;
        Ok(())
    }
    fn inner(from: &Path, to: &Path) -> io::Result<()> {
        match fs::rename(from, to) {
            Ok(_) => Ok(()),
            #[cfg(any())] // io_error_more is not stable yet
            Err(ref e) if e.kind() == io::ErrorKind::CrossesDevices => move_by_copy(from, to),
            Err(ref e) if e.raw_os_error() == Some(CROSSES_DEVICES_OS_CODE) => {
                move_by_copy(from, to)
            }
            Err(e) => Err(e),
        }
    }
    inner(from.as_ref(), to.as_ref())
}

fn local_low_appdata_path() -> &'static Path {
    static CELL: OnceBox<PathBuf> = OnceBox::new();
    CELL.get_or_init(|| {
        SHGetKnownFolderPath(&KNOWNFOLDERID::LocalAppDataLow, KF::DEFAULT, None)
            .map(PathBuf::from)
            .map(Box::new)
            .expect("getting LocalAppDataLow")
    })
}

fn config_file_path() -> &'static Path {
    static CELL: OnceBox<PathBuf> = OnceBox::new();
    /// returns read-writable file handle for config
    fn find_config_file() -> PathBuf {
        // first, find in exe folder
        if let Some(config_file) = std::env::current_exe()
            .ok()
            .and_then(|p| Some(p.parent()?.join("config.toml")))
            .take_if(|x| x.exists())
        {
            return config_file;
        }

        // then, create in LocalLow folder
        local_low_appdata_path().join("vrc-log-renamer/config.toml")
    }

    CELL.get_or_init(|| Box::new(find_config_file()))
}
