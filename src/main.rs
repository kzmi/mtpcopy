mod command_copy;
mod command_list_files;
mod command_list_storages;
mod copy;
mod finders;
mod glob;
mod path;
mod wpd;

use std::fmt::Write;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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
            "R",
            "recursive",
            "(with \"list\" command) list subfolders recursively",
        )
        .optflagmulti("v", "verbose", "verbose output.");

    let matches = options.parse(std::env::args().skip(1))?;

    let mut help = matches.opt_present("help");
    let version = matches.opt_present("version");
    let recursive = matches.opt_present("recursive");
    let verbose = matches.opt_count("verbose") as u32;

    let mut paths: Option<Paths> = None;
    let command: Command;

    if help || version {
        command = Command::None;
    } else if matches.free.len() > 0 {
        match find_command(&matches.free[0]) {
            None => {
                return Err(format!("unknwon command : {}", &matches.free[0]).into());
            }
            Some(cmd) => match cmd {
                Command::ListFiles => {
                    if matches.free.len() < 2 {
                        return Err("(command \"list\") pattern is not specified".into());
                    }
                    let src = String::from(&matches.free[1]);
                    let dest = String::from("");
                    paths = Some(Paths { src, dest });
                    command = cmd;
                }
                Command::Copy => {
                    if matches.free.len() < 2 {
                        return Err("(command \"copy\") source path is not specified".into());
                    }
                    if matches.free.len() < 3 {
                        return Err("(command \"copy\") destination path is not specified".into());
                    }
                    let src = String::from(&matches.free[1]);
                    let dest = String::from(&matches.free[2]);
                    paths = Some(Paths { src, dest });
                    command = cmd;
                }
                cmd => {
                    command = cmd;
                }
            },
        }
    } else {
        help = true;
        command = Command::None;
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
    write!(&mut s, "Usage: {} [-hV]\n", bin_name)?;
    write!(&mut s, "       {} copy <source-path> <dest-path>\n", bin_name)?;
    write!(&mut s, "       {} storages\n", bin_name)?;
    write!(&mut s, "       {} list [-Rv] <path>\n", bin_name)?;
    s.push_str("\n");
    s.push_str("Commands:\n");
    s.push_str("    copy       copy files or folders recursively.\n");
    s.push_str("               <dest-path> must be a path to the existing folder.\n");
    s.push_str("    storages   list all storages for the all connecting portable devices.\n");
    s.push_str("    list       list all file or folders matching the path.\n");
    s.push_str("               <path> can contains wildcard (see below.)\n");
    s.push_str("\n");
    s.push_str("About Path:\n");
    s.push_str("    A path on the portable device must be specified in this form:\n");
    s.push_str("        <device-name>:<storage-name>:<path>\n");
    s.push_str("        e.g. \"My Device:SD Card:\\Pictures\\2021\\April\"\n");
    s.push_str("\n");
    s.push_str("    In \"list\" command, the path can contain wildcard characters:\n");
    s.push_str("        e.g. \"My*:SD*:**\\2021\\**\\*.jpg\"\n");
    s.push_str("\n");
    s.push_str("    The other form will be used as the local path on your computer.");
    Ok(s)
}

fn show_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

fn find_command(s: &str) -> Option<Command> {
    let commands = [
        ("copy", Command::Copy),
        ("list", Command::ListFiles),
        ("storages", Command::ListStorages),
    ];
    let mut matched: Vec<Command> = commands
        .iter()
        .filter(|&&(kw, _)| match kw.find(s) {
            Some(n) if n == 0 => true,
            _ => false,
        })
        .map(|&(_, cmdval)| cmdval)
        .collect();

    if matched.len() == 1 {
        matched.pop()
    } else {
        None
    }
}
