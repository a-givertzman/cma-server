use std::env;

use once_cell::sync::Lazy;
use snap7_sys::LibSnap7;

static RED: &str = "\x1b[0;31m";
static NC: &str = "\x1b[0m"; // No Color

pub static S7LIB: Lazy<LibSnap7> = Lazy::new(|| {
    println!("initializing LibSnap7...");
    let paths = [
        format!("{}/libsnap7.so", env::current_dir().unwrap().display()),
        format!("/usr/lib/libsnap7.so"),
    ];
    for path in paths {
        println!("initializing LibSnap7 | check '{}'...", path);
        match unsafe { LibSnap7::new(&path) } {
            Ok(lib) => {
                println!("initializing LibSnap7 | check '{}' - ok", path);
                println!("initializing LibSnap7 - ok");
                return lib;
            },
            Err(_) => {
                println!("initializing LibSnap7 | check '{}' - {}not found{}", path, RED, NC);
            },
        }
    }
    println!("{}initializing LibSnap7 - ERROR{}", RED, NC);
    panic!("libsnap7.so - not found")
});

