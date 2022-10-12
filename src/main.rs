mod config;

use crate::config::{read_config, save_config};
use anyhow::{bail, Result};
use once_cell::race::OnceBox;
use std::path::{Path, PathBuf};
use take_if::TakeIf;
use winsafe::co::{DLGID, KF, KNOWNFOLDERID, MB};
use winsafe::prelude::user_Hwnd;
use winsafe::{SHGetKnownFolderPath, HWND};

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

    println!("config loaded: {:#?}", config);

    match save_config(&config) {
        Ok(()) => println!("config file written to: {}", config_file_path().display()),
        Err(e) => {
            eprintln!("error writing config: {:?}", e);
            let message = format!(
                "Error writing config file: {}.",
                e
            );
            HWND::GetDesktopWindow().MessageBox(&message, "Error", MB::OK)?;
            bail!(e);
        }
    };

    Ok(())
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
