mod command_list;
mod finders;
mod glob;
mod io;
mod wpd;

#[derive(Debug, Eq, PartialEq)]
enum Command {
    None,
    ListStorages,
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
}

fn main() {
    let result = run();
    if let Err(err) = result {
        eprintln!("{}", err.to_string());
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    match args.command {
        Command::ListStorages => command_list::command_list()?,
        _ => {}
    };
    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut options = getopts::Options::new();
    let result = options
        .optflag("h", "help", "show this help message.")
        .optflag("v", "version", "show the version and exit.")
        .optflag(
            "l",
            "list-storages",
            "list storages in the connected portable devices.",
        )
        .parse(std::env::args().skip(1));

    let mut matches;
    match result {
        Err(err) => {
            return Err(err.into());
        }
        Ok(m) => matches = m,
    }

    let mut help = matches.opt_present("help");
    let version = matches.opt_present("version");

    let mut command = if help || version {
        Command::None
    } else if matches.opt_present("list-storages") {
        Command::ListStorages
    } else {
        Command::Copy
    };

    let mut paths: Option<Paths> = None;

    if command == Command::Copy {
        if matches.free.len() == 0 {
            help = true;
            command = Command::None;
        } else if matches.free.len() == 1 {
            return Err("destination path is not specified.".into());
        } else if matches.free.len() == 2 {
            let dest = matches.free.pop().unwrap();
            let src = matches.free.pop().unwrap();
            paths = Some(Paths { src, dest });
        } else {
            return Err(format!("bad option : {}", &matches.free[2]).into());
        }
    }

    if help {
        let brief = usage_brief();
        show_version();
        print!("{}", options.usage(brief.as_str()));
    } else if version {
        show_version();
    }

    Ok(Args { command, paths })
}

fn usage_brief() -> String {
    let bin_name = env!("CARGO_BIN_NAME");
    String::new()
        + format!("Usage: {} [-hv]\n", bin_name).as_str()
        + format!("       {} -l\n", bin_name).as_str()
        + format!("       {} <source-path> <dest-path>\n", bin_name).as_str()
        + "\n"
        + "Path:\n"
        + "    A path on the portable device must be specified as:\n"
        + "        <device-name>:<storage-name>:<path>\n"
        + "        e.g. \"PD-123:SD Card:/Pictures/2021/April\"\n"
        + "        e.g. \"PD-???:*Card:/**/April\"\n"
        + "    The other will be used as the local path on your computer."
}

fn show_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
