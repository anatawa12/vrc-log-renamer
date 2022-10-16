// VRC Log Renamer / license gen: an utility to generate license list file
//
// MIT License
//
// Copyright (c) 2022 anatawa12
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use cargo_metadata::{MetadataCommand, PackageId};
use itertools::Itertools;
use std::borrow::Cow;
use std::fmt::Arguments;
use std::path::PathBuf;
use std::{fmt, fs, io};

#[derive(Default)]
pub struct Builder {
    name: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }
}

impl Builder {
    fn get_name(&self) -> Cow<str> {
        self.name.as_deref().map(Cow::Borrowed).unwrap_or_else(|| {
            Cow::Owned(std::env::var("CARGO_PKG_NAME").expect("no name specified"))
        })
    }

    #[allow(unused)]
    pub fn generate_to_io(&self, out: impl io::Write) -> io::Result<()> {
        generate(&mut IoWriteFmt(out), self.get_name())
    }

    #[allow(unused)]
    pub fn generate_to_fmt(&self, out: impl fmt::Write) -> fmt::Result {
        generate(&mut FmtWriteFmt(out), self.get_name())
    }

    #[allow(unused)]
    pub fn generate_to_string(&self, out: impl fmt::Write) -> Result<String, fmt::Error> {
        let mut str = String::new();
        self.generate_to_fmt(&mut str)?;
        Ok(str)
    }
}

trait WriteFmt {
    type Error;
    fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error>;
}

struct IoWriteFmt<W: io::Write>(W);
impl<W: io::Write> WriteFmt for IoWriteFmt<W> {
    type Error = io::Error;

    fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
        io::Write::write_fmt(&mut self.0, args)
    }
}

struct FmtWriteFmt<W: fmt::Write>(W);
impl<W: fmt::Write> WriteFmt for FmtWriteFmt<W> {
    type Error = fmt::Error;

    fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
        fmt::Write::write_fmt(&mut self.0, args)
    }
}

fn generate<W: WriteFmt>(out: &mut W, name: impl AsRef<str>) -> Result<(), W::Error> {
    let metadata = MetadataCommand::new().exec().unwrap();
    let mut packages = metadata
        .packages
        .into_iter()
        .map(|p| {
            let manifest_dir = p.manifest_path.parent().unwrap().as_std_path();
            PackageLicenseInfo {
                id: p.id,
                name: p.name,
                repository: p.repository.unwrap(),
                license_id: p.license.unwrap(),
                license_files: manifest_dir
                    .read_dir()
                    .expect("reading manifest dir")
                    .filter_ok(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .as_ref()
                            .to_ascii_lowercase()
                            .contains("license")
                    })
                    .map_ok(|e| e.path())
                    .collect::<Result<Vec<_>, _>>()
                    .expect("reading manifest dir"),
            }
        })
        .collect::<Vec<_>>();
    let root_id = metadata.resolve.unwrap().root.unwrap();
    let root_pkg = packages.swap_remove(packages.iter().position(|x| x.id == root_id).unwrap());
    packages.sort_by_key(|x| x.id.clone());

    writeln!(
        out,
        "This software {} is published under license of {}",
        name.as_ref(),
        root_pkg.license_id
    )?;
    writeln!(out, "and is hosted on {}.", root_pkg.repository)?;
    writeln!(out, "")?;
    print_license_list(out, "software", &root_pkg)?;
    for p in packages {
        let separator = "-".repeat(p.name.len() + (4 + 1) * 2);
        writeln!(out, "{}", separator)?;
        writeln!(out, "---- {} ----", p.name)?;
        writeln!(out, "{}", separator)?;
        writeln!(out, "")?;
        writeln!(
            out,
            "This software uses {} which is released under license of {}",
            p.name, p.license_id
        )?;
        writeln!(out, "which is hosted on {}.", p.repository)?;
        writeln!(out, "")?;
        print_license_list(out, "library", &p)?;
    }
    Ok(())
}

fn print_license_list<W: WriteFmt>(
    out: &mut W,
    kind: &str,
    p: &PackageLicenseInfo,
) -> Result<(), W::Error> {
    if p.license_files.is_empty() {
        writeln!(
            out,
            "This {} does not have any bundled license files.",
            kind
        )?;
        writeln!(out, "")?;
    } else {
        writeln!(out, "Here's list of license file bundled in this {}:", kind)?;
        writeln!(out, "")?;
        for x in &p.license_files {
            writeln!(
                out,
                "==== {} ====",
                x.file_name().unwrap().to_string_lossy()
            )?;
            writeln!(
                out,
                "{}",
                fs::read_to_string(x).expect("reading license file")
            )?;
        }
    }
    Ok(())
}

struct PackageLicenseInfo {
    id: PackageId,
    name: String,
    repository: String,
    license_id: String,
    license_files: Vec<PathBuf>,
}
