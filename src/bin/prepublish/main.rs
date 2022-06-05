use std::{env, fs, process};

use chrono::Utc;

use blog::post::PostSource;

fn main() {
    let mut failed = false;

    let paths = env::args_os()
        .skip(1);
    
    for path in paths {
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("failed to read {}: {}", path.to_string_lossy(), err);
                failed = true;
                continue
            }
        };

        let mut source = match contents.parse::<PostSource>() {
            Ok(source) => source,
            Err(err) => {
                eprintln!("failed to parse {}: {}", path.to_string_lossy(), err);
                failed = true;
                continue
            },
        };

        if source.header().published().is_none() {
            *source.header_mut().published_mut() = Some(Utc::now());
        }

        if let Err(err) = fs::write(&path, source.to_string()) {
            eprintln!("failed to write {}: {}", path.to_string_lossy(), err);
                failed = true;
                continue
        }
    }

    if failed {
        process::exit(1);
    }
}
