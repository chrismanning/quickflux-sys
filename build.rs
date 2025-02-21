#![feature(exit_status_error)]
/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::result::Result::*;
use semver::Version;

fn main() -> Result<(), Box<dyn Error>> {
    let quickflux_path = Path::new("vendor/quickflux").canonicalize()?;
    run_qmake(&quickflux_path)?;
    Command::new("make")
        .current_dir(&quickflux_path)
        .status()?
        .exit_ok()?;
    println!("cargo:rustc-link-search={}", quickflux_path.display());
    println!("cargo:rustc-link-lib=quickflux");

    let qt_include_path = std::env::var("DEP_QT_INCLUDE_PATH").unwrap();
    let qt_version = std::env::var("DEP_QT_VERSION")
        .unwrap()
        .parse::<Version>()
        .expect("Parsing Qt version failed");

    let mut config = cpp_build::Config::new();
    for f in std::env::var("DEP_QT_COMPILE_FLAGS").unwrap().split_terminator(";") {
        config.flag(f);
    }
    config
        .include("vendor/quickflux/src")
        .include(&qt_include_path)
        .include(format!("{}/QtCore", qt_include_path))
        .include(format!("{}/QtQml", qt_include_path))
        .build("src/lib.rs");

    for minor in 7..=15 {
        if qt_version >= Version::new(5, minor, 0) {
            println!("cargo:rustc-cfg=qt_{}_{}", 5, minor);
        }
    }
    let mut minor = 0;
    while qt_version >= Version::new(6, minor, 0) {
        println!("cargo:rustc-cfg=qt_{}_{}", 6, minor);
        minor += 1;
    }

    Ok(())
}

fn run_qmake(quickflux_path: &Path) -> Result<(), Box<dyn Error>> {
    let run = |qmake: &str| {
        Command::new(qmake)
            .arg("quickflux.pri")
            .arg("DEFINES=QUICK_FLUX_DISABLE_AUTO_QML_REGISTER")
            .current_dir(&quickflux_path)
            .status()?
            .exit_ok()?;
        Ok(())
    };
    match std::env::var("QMAKE") {
        Ok(qmake) => return run(&qmake),
        Err(_) => {
            for qmake in &["qmake", "qmake6", "qmake-qt5"] {
                if let Ok(_) = run(&qmake) {
                    return Ok(());
                }
            }
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
        }
    }
}
