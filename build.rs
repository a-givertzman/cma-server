use std::{fs, io::Write};

pub fn main() {
    build_slmp_module();
    println!(r"cargo:rustc-link-search=./lib/");
}
///
/// 
fn build_slmp_module() {
    log(format!("Build SLMP module | Clearing ..."));
    // let paths = scan_dir("./target/");
    // log(format!("Build SLMP module | Files found: {}", paths.len()));
    // for path in paths {
    //     log(format!("Build SLMP module | \tRemoving file '{}'", path));
    //     match fs::remove_file(&path) {
    //         Ok(_) => {
    //             log(format!("Removed file: {:?}", path));
    //         }
    //         Err(err) => {
    //             log(format!("Error removing file: {:?}: {}", path, err));
    //         }
    //     }
    // }
    // log(format!("Build SLMP module | Compiling ..."));
    cc::Build::new()
        .file("src/services/slmp_client/slmp/clib.c")
        // .flag("unwanted_flag")
        .compile("slmp");
    // log(format!("Build SLMP module | Coping ..."));
    // let paths = scan_dir("./target/");
    // log(format!("Build SLMP module | Files found: {}", paths.len()));
    // for path in paths {
    //     log(format!("Build SLMP module | \tCoping file '{}'", path));
    //     match fs::copy(&path, "./lib/") {
    //         Ok(_) => {
    //             log(format!("Copied file: {:?}", path));
    //         }
    //         Err(err) => {
    //             log(format!("Error cody file: {:?}: {}", path, err));
    //         }
    //     }
    // }
}

// fn scan_dir<PathAsRef: AsRef<Path>>(path: PathAsRef) -> Vec<String> {
//     let mut files = vec![];
//     if path.as_ref().is_dir() {
//         let paths = fs::read_dir(path).unwrap();
//         for path in paths {
//             match path {
//                 Ok(path) => {
//                     let path = path.path();
//                     if path.is_dir() {
//                         scan_dir(path);
//                     } else {
//                         if path.display().to_string().contains("slmp.a") {
//                             files.push(path.to_str().unwrap().to_owned());
//                             log(format!("!!! file: {:?}", path));
//                         }
//                     }
//                 }
//                 Err(_) => {},
//             }
//         }
//     }
//     files
// }

fn log(m: impl Into<String>) {
    let path = "logs/build.log";
    match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut f) => {
            let data: String = format!("{}\n", m.into());
            match f.write_all(data.as_bytes()) {
                Ok(_) => {
                    // debug!("{}.store | Retain stored in: {:?}", self.id, path);
                    // Ok(())
                }
                Err(_err) => {
                    // let message = format!("{}.store | Error writing to file: '{:?}'\n\terror: {:?}", self.id, path, err);
                    // error!("{}", message);
                    // Err(message)
                }
            }
        }
        Err(_err) => {
            // let message = format!("{}.store | Error open file: '{:?}'\n\terror: {:?}", self.id, path, err);
            // error!("{}", message);
            // Err(message)
        }
    }
}