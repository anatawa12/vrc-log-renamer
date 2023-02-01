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

use crate::{config_file_path, local_low_appdata_path};
use chrono::format::{Fixed, Item, Numeric, Pad, StrftimeItems};
use io::Error;
use regex::Regex;
use serde::ser::Error as _;
use serde::Serialize;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};
use toml::Value;

macro_rules! default_fns {
    ($value_name: ident: $ty: ty = $expr: expr; |$x: ident| $compare: expr) => {
        proc_macros::concat_ident! {
            fn $value_name ## _default_ref() -> &'static $ty {
                static CELL: once_cell::race::OnceBox<$ty> = once_cell::race::OnceBox::new();
                CELL.get_or_init(|| Box::new($expr))
            }

            fn is_ ## $value_name ## _default(v: &$ty) -> bool {
                (match v { $x => $compare }) == (match Self::$value_name ## _default_ref() { $x => $compare })
            }

            fn $value_name ## _default() -> $ty {
                Self::$value_name ## _default_ref().clone()
            }
        }
    };

    ($value_name: ident: $ty: ty = $expr: expr) => {
        default_fns!($value_name: $ty = $expr; |x| x);
    };
}

#[derive(Serialize, Debug, Clone)]
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
            self.output.read_from_file(output)?
        }
        Ok(())
    }

    pub fn new(source: Source, output: Output) -> Self {
        Self { source, output }
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

#[derive(Serialize, Debug, Clone)]
pub struct Source {
    #[serde(
        skip_serializing_if = "Source::is_folder_default",
        default = "Source::folder_default"
    )]
    folder: PathBuf,
    #[serde(
        skip_serializing_if = "Source::is_pattern_default",
        default = "Source::pattern_default",
        serialize_with = "serialize_regex"
    )]
    pattern: Regex,
    #[serde(
        skip_serializing_if = "Source::is_keep_old_default",
        default = "Source::keep_old_default"
    )]
    keep_old: bool,
}

fn serialize_regex<S: serde::Serializer>(regex: &Regex, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(regex.as_str())
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

    default_fns!(folder: PathBuf = local_low_appdata_path().join("VRChat").join("VRChat"));
    default_fns!(pattern: Regex = Regex::new(r#"^output_log_(?:\d{4}-\d{2}-\d{2}_)?\d{2}-\d{2}-\d{2}(?P<in_sec_num>\d+)?\.txt$"#).unwrap(); |x| x.as_str());
    default_fns!(keep_old: bool = true);

    pub fn folder(&self) -> &PathBuf {
        &self.folder
    }
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }
    pub fn keep_old(&self) -> bool {
        self.keep_old
    }

    pub fn new(folder: PathBuf, pattern: Regex, keep_old: bool) -> Self {
        Self {
            folder,
            pattern,
            keep_old,
        }
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

#[derive(Serialize, Debug, Clone)]
pub struct Output {
    #[serde(
        skip_serializing_if = "Output::is_folder_default",
        default = "Output::folder_default"
    )]
    folder: PathBuf,
    #[serde(
        skip_serializing_if = "Output::is_pattern_default",
        default = "Output::pattern_default",
        serialize_with = "serialize_pattern"
    )]
    pattern: Vec<Item<'static>>,
    #[serde(
        skip_serializing_if = "Output::is_utc_time_default",
        default = "Output::utc_time_default"
    )]
    utc_time: bool,
    #[serde(
        skip_serializing_if = "Output::is_file_ctime_default",
        default = "Output::file_ctime_default"
    )]
    file_ctime: bool,
}

fn format_internal_format(fixed: &chrono::format::InternalFixed) -> Option<&'static str> {
    use chrono::format::InternalFixed;
    use once_cell::race::OnceBox;
    type MappingType = [(InternalFixed, &'static str); 3];
    static MAPPING: OnceBox<MappingType> = OnceBox::new();
    fn init_mapping() -> Box<MappingType> {
        fn fixed_internal(format: &str) -> (InternalFixed, &str) {
            match StrftimeItems::new(format).next().unwrap() {
                Item::Fixed(Fixed::Internal(fixed)) => (fixed, format),
                _ => unreachable!("fixed_internal init failed"),
            }
        }
        Box::new([
            fixed_internal("%3f"),
            fixed_internal("%6f"),
            fixed_internal("%9f"),
        ])
    }
    MAPPING.get_or_init(init_mapping)
        .iter()
        .find(|(pat, _)| pat == fixed)
        .map(|(_, a)| *a)
}

fn pattern_to_string(pattern: &Vec<Item<'static>>) -> Result<String, &'static str> {
    let mut string = String::new();
    for x in pattern {
        match x {
            Item::Literal(s) => string.push_str(s),
            Item::OwnedLiteral(s) => string.push_str(s),
            Item::Space(s) => string.push_str(s),
            Item::OwnedSpace(s) => string.push_str(s),
            Item::Numeric(n, p) => {
                string.push('%');
                match p {
                    Pad::None => string.push('-'),
                    Pad::Zero => string.push('0'),
                    Pad::Space => string.push('_'),
                }
                // see https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                match n {
                    // DATE SPECIFIERS:
                    Numeric::Year => string.push('Y'),
                    Numeric::YearDiv100 => string.push('C'),
                    Numeric::YearMod100 => string.push('y'),

                    Numeric::Month => string.push('m'),
                    // month name 3 letters: b = h
                    // month name N letters: B
                    Numeric::Day => string.push('d'),
                    // %e = %_d

                    // weekday name 3 letters: a
                    // weekday name N letters: A
                    Numeric::WeekFromSun => string.push('w'),
                    Numeric::WeekFromMon => string.push('u'),

                    Numeric::NumDaysFromSun => string.push('U'),
                    Numeric::WeekdayFromMon => string.push('W'),

                    Numeric::IsoYear => string.push('G'),
                    Numeric::IsoYearMod100 => string.push('g'),
                    Numeric::IsoWeek => string.push('V'),

                    Numeric::Ordinal => string.push('j'),

                    Numeric::IsoYearDiv100 => string.push('g'), // unknown

                    // time specifiers
                    Numeric::Hour => string.push('H'),
                    // %k = %_H
                    Numeric::Hour12 => string.push('I'),
                    // %l = %_I

                    // am/pm: P
                    // AM/PM: p
                    Numeric::Minute => string.push('M'),
                    Numeric::Second => string.push('S'),
                    Numeric::Nanosecond => string.push('f'),
                    // %.{3,6,9,}f and %{3,6,9}f

                    // TIME ZONE SPECIFIERS and DATE & TIME SPECIFIERS
                    Numeric::Timestamp => string.push('s'),

                    Numeric::Internal(_) => return Err("internal format found"),
                }
            }
            Item::Fixed(f) => match f {
                Fixed::ShortMonthName => string.push_str("%b"),
                Fixed::LongMonthName => string.push_str("%B"),
                Fixed::ShortWeekdayName => string.push_str("%a"),
                Fixed::LongWeekdayName => string.push_str("%A"),
                Fixed::LowerAmPm => string.push_str("%P"),
                Fixed::UpperAmPm => string.push_str("%p"),
                Fixed::Nanosecond => string.push_str("%.f"),
                Fixed::Nanosecond3 => string.push_str("%.3f"),
                Fixed::Nanosecond6 => string.push_str("%.6f"),
                Fixed::Nanosecond9 => string.push_str("%.9f"),
                Fixed::TimezoneName => string.push_str("%Z"),
                Fixed::TimezoneOffsetColon => string.push_str("%:z"),
                Fixed::TimezoneOffset => string.push_str("%z"),
                Fixed::RFC2822 => string.push_str("%c"),
                Fixed::RFC3339 => string.push_str("%+"),
                Fixed::TimezoneOffsetColonZ => return Err("internal format found"),
                Fixed::TimezoneOffsetZ => return Err("internal format found"),
                Fixed::Internal(format_in) => {
                    string.push_str(format_internal_format(format_in).ok_or("internal format found")?);
                }
            },
            Item::Error => return Err("format error found"),
        }
    }
    return Ok(string);
}

fn serialize_pattern<S: serde::Serializer>(
    pattern: &Vec<Item<'static>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_str(&pattern_to_string(pattern).map_err(S::Error::custom)?)
}

pub fn parse_pattern(str: &str) -> Option<Vec<Item<'static>>> {
    fn own_strftime(item: Item) -> Item<'static> {
        match item {
            Item::Literal(s) => Item::OwnedLiteral(s.to_string().into_boxed_str()),
            Item::Space(s) => Item::OwnedSpace(s.to_string().into_boxed_str()),
            Item::OwnedLiteral(s) => Item::OwnedLiteral(s),
            Item::OwnedSpace(s) => Item::OwnedSpace(s),
            Item::Numeric(n, p) => {
                if matches!(n, Numeric::Internal(_)) {
                    // internal format is not allowed
                    Item::Error
                } else {
                    Item::Numeric(n, p)
                }
            }
            Item::Fixed(f) => {
                match f {
                    Fixed::Internal(internal) => {
                        if format_internal_format(&internal).is_some() {
                            Item::Fixed(Fixed::Internal(internal))
                        } else {
                            Item::Error
                        }
                    }
                    Fixed::TimezoneOffset | Fixed::TimezoneOffsetZ => Item::Error,
                    f => Item::Fixed(f)
                }
            }
            Item::Error => Item::Error,
        }
    }
    let pattern: Vec<Item> = StrftimeItems::new(str).map(own_strftime).collect();
    if pattern.iter().any(|x| matches!(x, Item::Error)) {
        return None;
    }
    Some(pattern)
}

impl Output {
    default_fns!(
        folder: PathBuf = local_low_appdata_path()
            .join("VRChat")
            .join("VRChat")
            .join("logs")
    );
    default_fns!(
        pattern: Vec<Item<'static>> = StrftimeItems::new("output_log_%Y-%m-%d_%H-%M-%S{regex:in_sec_num}.txt").collect::<Vec<_>>();
        |x| pattern_to_string(x)
    );
    default_fns!(utc_time: bool = false);
    default_fns!(file_ctime: bool = false);

    pub(crate) fn read_from_file(&mut self, toml: &Value) -> io::Result<()> {
        if let Some(Value::String(str)) = toml.get("folder") {
            self.folder = PathBuf::from(str)
        }
        if let Some(Value::String(str)) = toml.get("pattern") {
            // previously, skip_serializing_if = "Output::is_pattern_default" is not working well.
            static TRADITIONAL_DEFAULT: &str = "output_log_%0Y-%0m-%0d_%0H-%0M-%0S.txt";
            if str != TRADITIONAL_DEFAULT {
                self.pattern = parse_pattern(&str).ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("'{}' is invalid log file pattern", str),
                    )
                })?;
            }
        }
        if let Some(Value::Boolean(bool)) = toml.get("utc_time") {
            self.utc_time = *bool;
        }
        if let Some(Value::Boolean(bool)) = toml.get("file_ctime") {
            self.file_ctime = *bool;
        }
        Ok(())
    }

    pub fn folder(&self) -> &PathBuf {
        &self.folder
    }

    pub fn pattern(&self) -> &Vec<Item<'static>> {
        &self.pattern
    }

    pub fn pattern_as_string(&self) -> String {
        pattern_to_string(&self.pattern).unwrap()
    }

    pub fn utc_time(&self) -> bool {
        self.utc_time
    }

    pub fn file_ctime(&self) -> bool {
        self.file_ctime
    }

    pub fn new(
        folder: PathBuf,
        pattern: Vec<Item<'static>>,
        utc_time: bool,
        file_ctime: bool,
    ) -> Self {
        Self {
            folder,
            pattern,
            utc_time,
            file_ctime,
        }
    }
}

impl Default for Output {
    fn default() -> Self {
        Self {
            folder: Self::folder_default(),
            pattern: Self::pattern_default(),
            utc_time: Self::utc_time_default(),
            file_ctime: Self::file_ctime_default(),
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

pub fn save_config(config: &ConfigFile) -> io::Result<()> {
    fs::create_dir_all(config_file_path().parent().unwrap())?;
    fs::write(
        config_file_path(),
        toml::to_string(config).map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
    )
}
