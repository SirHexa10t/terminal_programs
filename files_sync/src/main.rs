use clap::{ArgAction, ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Sync directories by tracking and comparing directory trees",
    after_help = r#"EXAMPLES:
  filesync -t "$HOME/Downloads"
  filesync -t "$HOME/Downloads" -o firefox_pictures
  filesync -d "$HOME/Downloads" "$HOME/Pictures"
  filesync -s "$HOME/Downloads" "$HOME/Pictures" --dry-run
"#
)]
#[command(
    group(
        ArgGroup::new("command")
            .required(true)
            .multiple(false) // exactly ONE of these must be present
            .args(["track", "diff", "sync"])
    )
)]
struct Args {
    /// Write a tracking file to PATH (requires a DIR positional argument)
    #[arg(short = 't', long = "track", value_name = "DIR")]
    track: Option<PathBuf>,

    /// Compare master vs slave directories
    #[arg(short = 'd', long = "diff", value_names = ["DIR_MASTER", "DIR_SLAVE"], num_args = 2)]
    diff: Option<Vec<PathBuf>>,

    /// Sync slave directory to match master directory
    #[arg(short = 's', long = "sync", value_names = ["DIR_MASTER", "DIR_SLAVE"], num_args = 2)]
    sync: Option<Vec<PathBuf>>,


    //optionals:

    /// Only include paths under SUBDIR (repeatable). Allowed in any mode.
    #[arg(short = 'o', long = "only", value_name = "SUBDIR", action = ArgAction::Append)]
    only: Vec<PathBuf>,

    /// Print actions only (valid with --sync)
    #[arg(long, requires = "sync")]
    dry_run: bool,

}


fn main() {
    let args = Args::parse();

    if let Some(dir) = args.track {
        filesync::write_tracking_file_with_listing(dir);
    } else if let Some(v) = args.diff {
        let master = &v[0];
        let slave = &v[1];
        // ...
    } else if let Some(v) = args.sync {
        let master = &v[0];
        let slave = &v[1];
        // ...
    } else {
        unreachable!("clap ArgGroup enforces exactly one command");
    }
}
