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

use license_gen::Builder;
use std::path::PathBuf;
use std::{env, fs, io};

fn main() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    res.set_windres_path("x86_64-w64-mingw32-windres");
    res.set("InternalName", "TEST.EXE");
    res.compile()?;

    Builder::new().generate_to_io(fs::File::create(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("licenses.txt"),
    )?)?;

    Ok(())
}
