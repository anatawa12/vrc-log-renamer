use std::{io};

fn main() -> io::Result<()> {
    let mut res = winres::WindowsResource::new();
    res.set_windres_path("x86_64-w64-mingw32-windres");
    res.set("InternalName", "TEST.EXE");
    res.compile()?;
    Ok(())
}
