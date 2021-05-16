mod command_copy;
mod command_list_files;
mod command_list_storages;
mod copy;
mod finders;
mod glob;
mod path;
mod wpd;

use std::fmt::Write;

#[derive(Debug, Eq, PartialEq)]
enum Command {
    None,
    ListStorages,
    ListFiles,
    Copy,
}

#[derive(Debug)]
pub struct Paths {
    src: String,
    dest: String,
}

#[derive(Debug)]
struct Args {
    command: Command,
    paths: Option<Paths>,
    recursive: bool,
    verbose: u32,
}

fn main() {
    pretty_env_logger::init();
    let result = run();
    if let Err(err) = result {
        log::error!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    match args.command {
        Command::ListStorages => command_list_storages::command_list_storages()?,

        Command::ListFiles => command_list_files::command_list_files(
            args.paths.unwrap().src,
            args.recursive,
            args.verbose,
        )?,

        Command::Copy => command_copy::command_copy(&args.paths.unwrap())?,
        _ => {}
    };
    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut options = getopts::Options::new();
    options
        .optflag("h", "help", "show this help message.")
        .optflag("V", "version", "show version and exit.")
        .optflag(
            "s",
            "storages",
            "list all storages on the connected portable devices.",
        )
        .optflag("l", "list", "list files on the connected portable devices.")
        .optflag("R", "recursive", "(with -l) list subfolders recursively")
        .optflagmulti("v", "verbose", "verbose output.");
    let mut matches = options.parse(std::env::args().skip(1))?;

    let mut help = matches.opt_present("help");
    let version = matches.opt_present("version");
    let recursive = matches.opt_present("recursive");
    let verbose = matches.opt_count("verbose") as u32;

    let mut paths: Option<Paths> = None;
    let command: Command;
    if help || version {
        command = Command::None;
    } else if matches.opt_present("storages") {
        command = Command::ListStorages;
    } else if matches.opt_present("list") {
        if matches.free.len() == 0 {
            help = true;
            command = Command::None;
        } else if matches.free.len() == 1 {
            let src = matches.free.pop().unwrap();
            let dest = "".to_string();
            paths = Some(Paths { src, dest });
            command = Command::ListFiles;
        } else {
            return Err(format!("bad option : {}", &matches.free[1]).into());
        }
    } else {
        if matches.free.len() == 0 {
            help = true;
            command = Command::None;
        } else if matches.free.len() == 1 {
            return Err("destination path is not specified.".into());
        } else if matches.free.len() == 2 {
            let dest = matches.free.pop().unwrap();
            let src = matches.free.pop().unwrap();
            paths = Some(Paths { src, dest });
            command = Command::Copy;
        } else {
            return Err(format!("bad option : {}", &matches.free[2]).into());
        }
    }

    if help {
        let brief = usage_brief()?;
        show_version();
        print!("{}", options.usage(&brief));
    } else if version {
        show_version();
    }

    Ok(Args {
        command,
        paths,
        recursive,
        verbose,
    })
}

fn usage_brief() -> Result<String, std::fmt::Error> {
    let bin_name = env!("CARGO_BIN_NAME");
    let mut s = String::new();
    write!(&mut s, "Usage: {} [-hv]\n", bin_name)?;
    write!(&mut s, "       {} [-s]\n", bin_name)?;
    write!(&mut s, "       {} [-l] [-Rv] <path>\n", bin_name)?;
    write!(&mut s, "       {} <source-path> <dest-path>\n", bin_name)?;
    s.push_str("\n");
    s.push_str("Path:\n");
    s.push_str("    A path on the portable device must be specified as:\n");
    s.push_str("        <device-name>:<storage-name>:<path>\n");
    s.push_str("        e.g. \"PD-123:SD Card:/Pictures/2021/April\"\n");
    s.push_str("        e.g. \"PD-???:*Card:/**/April\"\n");
    s.push_str("    The other will be used as the local path on your computer.");
    Ok(s)
}

fn show_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
