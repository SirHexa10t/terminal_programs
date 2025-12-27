use std::env;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let target_dir = match args.next() {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("Usage: filesync <DIR>");
            std::process::exit(2);
        }
    };

    if args.next().is_some() {
        eprintln!("Usage: filesync <DIR>");
        std::process::exit(2);
    }

    files_sync::write_tracking_file(&target_dir);
    Ok(())
}
