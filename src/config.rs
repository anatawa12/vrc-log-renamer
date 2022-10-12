use crate::{config_file_path, local_low_appdata_path};
use chrono::format::{Fixed, Item, Numeric, Pad, StrftimeItems};
use io::Error;
use regex::Regex;
use serde::ser::Error as _;
use serde::Serialize;
use std::io::ErrorKind;
use std::path::{is_separator, PathBuf};
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
    default_fns!(pattern: Regex = Regex::new(r#"output_log_\d{2}-\d{2}-\d{2}\.txt"#).unwrap(); |x| x.as_str());
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
}

fn serialize_pattern<S: serde::Serializer>(
    pattern: &Vec<Item<'static>>,
    s: S,
) -> Result<S::Ok, S::Error> {
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

                    Numeric::Internal(_) => return Err(S::Error::custom("internal format found")),
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
                Fixed::Nanosecond3 => string.push_str("%3.f"),
                Fixed::Nanosecond6 => string.push_str("%6.f"),
                Fixed::Nanosecond9 => string.push_str("%9.f"),
                Fixed::TimezoneName => string.push_str("%Z"),
                Fixed::TimezoneOffsetColon => string.push_str("%:z"),
                Fixed::TimezoneOffset => string.push_str("%z"),
                Fixed::RFC2822 => string.push_str("%c"),
                Fixed::RFC3339 => string.push_str("%+"),
                Fixed::TimezoneOffsetColonZ => return Err(S::Error::custom("internal format found")),
                Fixed::TimezoneOffsetZ => return Err(S::Error::custom("internal format found")),
                Fixed::Internal(_) => return Err(S::Error::custom("internal format found")),
            },
            Item::Error => return Err(S::Error::custom("format error found")),
        }
    }
    s.serialize_str(&string)
}

impl Output {
    default_fns!(
        folder: PathBuf = local_low_appdata_path()
            .join("VRChat")
            .join("VRChat")
            .join("logs")
    );
    default_fns!(
        pattern: Vec<Item<'static>> =
            StrftimeItems::new("output_log_%Y-%m-%d_%H-%M-%S.txt").collect::<Vec<_>>()
    );
    default_fns!(utc_time: bool = false);

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
                    Item::Numeric(n, p) => {
                        if matches!(n, Numeric::Internal(_)) {
                            // internal format is not allowed
                            Item::Error
                        } else {
                            Item::Numeric(n, p)
                        }
                    }
                    Item::Fixed(f) => {
                        if matches!(
                            f,
                            Fixed::Internal(_)
                                | Fixed::TimezoneOffsetColonZ
                                | Fixed::TimezoneOffsetZ
                        ) {
                            // internal and -Z format is not allowed
                            Item::Error
                        } else {
                            Item::Fixed(f)
                        }
                    }
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

pub fn save_config(config: &ConfigFile) -> io::Result<()> {
    fs::write(config_file_path(), toml::to_string(config)?)
}
