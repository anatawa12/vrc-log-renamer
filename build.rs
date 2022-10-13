use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

fn main() {
    const RES: &str = "resources/resources.res";
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", RES);

    copy_resources(&out_dir, Path::new("resources/resources.res"));
}

fn copy_resources(out_dir: &Path, res: &Path) {
    match env::var("CARGO_CFG_TARGET_ENV").unwrap().as_str() {
        "msvc" => {
            let mut dotres = out_dir.join("resources.res");
            let mut dotlib = out_dir.join("resources.res.lib");

            fs::copy(res, &dotlib).unwrap();

            println!("cargo:rustc-link-search={}", out_dir.to_string_lossy());
            println!("cargo:rustc-link-lib=static={}", "resources.res");
        }
        "gnu" => {
            let prefix = env::var("RUSTC_LINKER")
                .as_ref()
                .map(String::as_str)
                .unwrap_or("x86_64-w64-mingw32-gcc")
                .trim_end_matches("gcc")
                .to_string();
            let windres = format!("{}windres", prefix);
            let mut lib_name = OsString::new();
            lib_name.push("lib");
            lib_name.push(res.file_name().unwrap());
            lib_name.push(".a");
            let object = out_dir.join(lib_name);
            //panic!("windres: {}", windres);
            let exit = Command::new(windres)
                .arg("-Ocoff")
                .arg("-v")
                .arg("-c65001")
                .arg("-i")
                .arg(res)
                .arg("-o")
                .arg(&object)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("windres not launched");
            //.exit_ok()
            //.unwrap();
            assert!(exit.success());
            println!(
                "cargo:rustc-link-search={}",
                object.parent().unwrap().to_string_lossy()
            );
            println!("cargo:rustc-link-lib=static={}", "resources.res");
        }
        _ => panic!("unsupported target env. gnu or msvc required"),
    }
}
