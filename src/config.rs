use crate::{config_file_path, local_low_appdata_path};
use chrono::format::{Item, StrftimeItems};
use io::Error;
use regex::Regex;
use serde::Serialize;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};
use toml::Value;

#[derive(Serialize, Debug)]
pub struct ConfigFile {
    source: Source,
    output: Output,
}

impl ConfigFile {
    fn read_from_file(&mut self, toml: &Value) -> io::Result<()> {
        if let Some(source) = toml.get("source") {
            self.source.read_from_file(source)?
        }
        if let Some(output) = toml.get("output") {
            self.source.read_from_file(output)?
        }
        Ok(())
    }
}

impl ConfigFile {
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub fn output(&self) -> &Output {
        &self.output
    }
}

#[derive(Serialize, Debug)]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none", default = "Source::folder_default")]
    folder: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none", default = "Source::pattern_default")]
    pattern: Regex,
    #[serde(skip_serializing_if = "Option::is_none", default = "Source::keep_old_default")]
    keep_old: bool,
}

impl Source {
    pub(crate) fn read_from_file(&mut self, toml: &Value) -> io::Result<()> {
        if let Some(Value::String(str)) = toml.get("folder") {
            self.folder = PathBuf::from(str)
        }
        if let Some(Value::String(str)) = toml.get("pattern") {
            self.pattern = Regex::new(str).map_err(|e| Error::new(ErrorKind::InvalidData, e))?
        }
        if let Some(Value::Boolean(bool)) = toml.get("keep_old") {
            self.keep_old = *bool;
        }
        Ok(())
    }

    fn folder_default() -> PathBuf {
        local_low_appdata_path().join("VRChat").join("VRChat")
    }
    fn pattern_default() -> Regex {
        Regex::new(r#"output_log_\d{2}-\d{2}-\d{2}\.txt"#).unwrap()
    }
    fn keep_old_default() -> bool {
        true
    }

    pub fn folder(&self) -> &PathBuf {
        &self.folder
    }
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }
    pub fn keep_old(&self) -> bool {
        self.keep_old
    }
}

impl Default for Source {
    fn default() -> Self {
        Source {
            folder: Self::folder_default(),
            pattern: Self::pattern_default(),
            keep_old: Self::keep_old_default(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Output {
    #[serde(skip_serializing_if = "Option::is_none", default = "Output::folder_default")]
    folder: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none", default = "Output::pattern_default")]
    pattern: Vec<Item<'static>>,
    #[serde(skip_serializing_if = "Option::is_none", default = "Output::utc_time_default")]
    utc_time: bool,
}

impl Output {
    fn folder_default() -> PathBuf {
        local_low_appdata_path()
            .join("VRChat")
            .join("VRChat")
            .join("logs")
    }
    fn pattern_default() -> Vec<Item<'static>> {
        StrftimeItems::new("output_log_%Y-%m-%d_%H-%M-%S.txt").collect::<Vec<_>>()
    }
    fn utc_time_default() -> bool {
        false
    }

    pub(crate) fn read_from_file(&mut self, toml: &Value) -> io::Result<()> {
        if let Some(Value::String(str)) = toml.get("folder") {
            self.folder = PathBuf::from(str)
        }
        if let Some(Value::String(str)) = toml.get("pattern") {
            fn own_strftime(item: Item) -> Item<'static> {
                match item {
                    Item::Literal(s) => Item::OwnedLiteral(s.to_string().into_boxed_str()),
                    Item::Space(s) => Item::OwnedSpace(s.to_string().into_boxed_str()),
                    Item::OwnedLiteral(s) => Item::OwnedLiteral(s),
                    Item::OwnedSpace(s) => Item::OwnedSpace(s),
                    Item::Numeric(n, p) => Item::Numeric(n, p),
                    Item::Fixed(f) => Item::Fixed(f),
                    Item::Error => Item::Error,
                }
            }
            self.pattern = StrftimeItems::new(str).map(own_strftime).collect();
            if self.pattern.iter().any(|x| matches!(x, Item::Error)) {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("'{}' is invalid log file pattern", str),
                ));
            }
        }
        if let Some(Value::Boolean(bool)) = toml.get("utc_time") {
            self.utc_time = *bool;
        }
        Ok(())
    }

    pub fn folder(&self) -> &PathBuf {
        &self.folder
    }

    pub fn pattern(&self) -> &Vec<Item<'static>> {
        &self.pattern
    }

    pub fn utc_time(&self) -> bool {
        self.utc_time
    }
}

impl Default for Output {
    fn default() -> Self {
        Self {
            folder: Self::folder_default(),
            pattern: Self::pattern_default(),
            utc_time: Self::utc_time_default(),
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            source: Default::default(),
            output: Default::default(),
        }
    }
}

pub fn read_config() -> io::Result<ConfigFile> {
    let mut config = ConfigFile::default();
    match fs::read_to_string(config_file_path()) {
        Ok(toml) => config.read_from_file(&toml::from_str::<Value>(&toml)?)?,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }

    Ok(config)
}
