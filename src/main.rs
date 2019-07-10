use clap::{App, Arg, SubCommand};
use llg_companion::Config;

fn main() {
    let mut app = App::new("llgCompanion")
        .version(env!("CARGO_PKG_VERSION")) // load version from cargo
        .author("Finnens <me@kloenk.de>")
        .about("llg companion server")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("set config file")
                .takes_value(true)
                .default_value("config.toml"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("dsb.url")
                .long("dsb.url")
                .help("set dsb url")
                .takes_value(true)
                .hidden(true)
                .value_name("dbs"),
        )
        .arg(
            Arg::with_name("dsb.userid")
                .long("dsb.userid")
                .help("set userid for dsb")
                .takes_value(true)
                .value_name("USERID"),
        )
        .arg(
            Arg::with_name("dsb.cookie")
                .long("dsb.cookie")
                .help("set dsb cookie for auth")
                .takes_value(true)
                .value_name("COOKIE"),
        )
        .arg(
            Arg::with_name("dsb.password")
                .long("dsb.password")
                .help("set dsb password")
                .takes_value(true)
                .value_name("PASSWORD"),
        )
        .arg(
            Arg::with_name("planinfo.baseurl")
                .long("plainfo.baseurl")
                .help("set planinfo baseurl")
                .takes_value(true)
                .hidden(true)
                .value_name("URL"),
        )
        .arg(
            Arg::with_name("planinfo.schoolid")
                .long("planinfo.schoolid")
                .help("set schoolid for planinfo")
                .takes_value(true)
                .value_name("ID"),
        )
        .arg(
            Arg::with_name("planinfo.cookie")
                .long("planinfo.cookie")
                .help("set cookies for planinfo")
                .takes_value(true)
                .value_name("COOKIE"),
        )
        .arg(
            Arg::with_name("impressum")
                .long("impressum")
                .short("i")
                .help("set url of impressum")
                .takes_value(true)
                .value_name("URL"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .short("p")
                .help("set port")
                .takes_value(true)
                .value_name("port"),
        )
        .arg(
            Arg::with_name("address")
                .long("interface")
                .short("I")
                .help("set interface to listen on")
                .takes_value(true)
                .value_name("ADDRESS"),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("create completions")
                .version("0.1.0")
                .author("Finn Behrens <me@kloenk.de>")
                .arg(
                    Arg::with_name("shell")
                        .help("set the shell to create for. Tries to identify with env variable")
                        .index(1)
                        .required(false)
                        .value_name("SHELL")
                        .possible_value("fish")
                        .possible_value("bash")
                        .possible_value("zsh")
                        .possible_value("powershell")
                        .possible_value("elvish"),
                )
                .arg(
                    Arg::with_name("out")
                        .help("sets output file")
                        .value_name("FILE")
                        .short("o")
                        .long("output"),
                )
                .setting(clap::AppSettings::ColorAuto)
                .setting(clap::AppSettings::ColoredHelp),
        )
        .setting(clap::AppSettings::ColorAuto)
        .setting(clap::AppSettings::ColoredHelp);

    let matches = app.clone().get_matches();

    // run subcommands
    if let Some(matches) = matches.subcommand_matches("completion") {
        completion(&matches, &mut app);
        std::process::exit(0);
    }
    drop(app);

    // Gets a value for config if supplied by user, or defaults to "config.toml"
    let config = matches.value_of("config").unwrap_or("config.toml");
    println!("Value for config: {}", config);

    let config: Option<toml::Value> = match std::fs::read_to_string(config) {
        Ok(config) => match toml::from_str(config.as_str()) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("Error parsing config file: {}", err);
                None
            }
        },
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            None
        }
    };

    let mut conf = Config::new();

    // read verbose value
    conf.verbose = {
        let mut a = matches.occurrences_of("verbose") as u8;
        let mut b = 0;
        if let Some(config) = &config {
            b = match config.get("verbose") {
                Some(verbose) => match verbose.as_integer() {
                    Some(verbose) => verbose as u8,
                    None => 0,
                },
                None => 0 as u8,
            };
            println!("{}", b);
        }
        if b > a {
            a = b;
        }
        a
    };
    conf.dsb.verbose = conf.verbose;

    if let Some(user_id) = &matches.value_of("dsb.userid") {
        conf.dsb.user_id = user_id.to_string();
    } else if let Some(config) = &config {
        if let Some(dsb) = config.get("dsb") {
            if let Some(user_id) = dsb.get("userid") {
                if let Some(user_id) = user_id.as_str() {
                    conf.dsb.user_id = user_id.to_string();
                }
            }
        }
    }

    if let Some(password) = &matches.value_of("dsb.password") {
        conf.dsb.password = password.to_string();
    } else if let Some(config) = &config {
        if let Some(dsb) = config.get("dsb") {
            if let Some(password) = dsb.get("password") {
                if let Some(password) = password.as_str() {
                    conf.dsb.password = password.to_string();
                }
            }
        }
    }

    if let Some(cookie) = &matches.value_of("dsb.cookie") {
        conf.dsb.cookie = cookie.to_string();
    } else if let Some(config) = &config {
        if let Some(dsb) = config.get("dsb") {
            if let Some(cookie) = dsb.get("cookie") {
                if let Some(cookie) = cookie.as_str() {
                    conf.dsb.cookie = cookie.to_string();
                }
            }
        }
    }

    if let Some(url) = &matches.value_of("dsb.url") {
        conf.dsb.url = url.to_string();
    } else if let Some(config) = &config {
        if let Some(dsb) = config.get("dsb") {
            if let Some(url) = dsb.get("url") {
                if let Some(url) = url.as_str() {
                    conf.dsb.url = url.to_string();
                }
            }
        }
    }

    if let Some(url) = &matches.value_of("planinfo.baseurl") {
        conf.planino.base_url = url.to_string();
    } else if let Some(config) = &config {
        if let Some(planinfo) = config.get("planinfo") {
            if let Some(url) = planinfo.get("url") {
                if let Some(url) = url.as_str() {
                    conf.planino.base_url = url.to_string();
                }
            }
        }
    }

    if let Some(schoolid) = &matches.value_of("planinfo.schoolid") {
        conf.planino.school_id = schoolid.to_string();
    } else if let Some(config) = &config {
        if let Some(planinfo) = config.get("planinfo") {
            if let Some(schoolid) = planinfo.get("schoolid") {
                if let Some(url) = schoolid.as_str() {
                    conf.planino.school_id = url.to_string();
                }
            }
        }
    }

    if let Some(cookie) = &matches.value_of("planinfo.cookie") {
        conf.planino.cookies = cookie.to_string();
    } else if let Some(config) = &config {
        if let Some(planinfo) = config.get("planinfo") {
            if let Some(cookie) = planinfo.get("url") {
                if let Some(cookie) = cookie.as_str() {
                    conf.planino.cookies = cookie.to_string();
                }
            }
        }
    }

    if let Some(impressum) = &matches.value_of("impressum") {
        conf.impressum = impressum.to_string();
    } else if let Some(config) = &config {
        if let Some(impressum) = config.get("impressum") {
            if let Some(impressum) = impressum.as_str() {
                conf.impressum = impressum.to_string();
            }
        }
    }

    if let Some(port) = &matches.value_of("port") {
        conf.port = port.parse().unwrap_or(conf.port);
    } else if let Some(config) = &config {
        if let Some(port) = config.get("port") {
            if let Some(port) = port.as_integer() {
                conf.port = port as u16;
            }
        }
    }

    if let Some(address) = &matches.value_of("address") {
        conf.address = address.to_string();
    } else if let Some(config) = &config {
        if let Some(address) = config.get("address") {
            if let Some(address) = address.as_str() {
                conf.address = address.to_string();
            }
        }
    }

    if conf.verbose >= 1 {
        println!("run llgCompanion on {}:{}", conf.address, conf.port);
    }
    if conf.verbose >= 2 {
        println!("Debug{}: enabled", conf.verbose);
    }

    conf.run().unwrap(); // FIXME: unwrap()
}

// create completion
fn completion(args: &clap::ArgMatches, app: &mut App) {
    let shell: String = match args.value_of("shell") {
        Some(shell) => shell.to_string(),
        None => {
            let shell = match std::env::var("SHELL") {
                Ok(shell) => shell,
                Err(_) => "/bin/bash".to_string(),
            };
            let shell = std::path::Path::new(&shell);
            match shell.file_name() {
                Some(shell) => shell.to_os_string().to_string_lossy().to_string(),
                None => "bash".to_string(),
            }
        }
    };

    use clap::Shell;
    let shell_l = shell.to_lowercase();
    let shell: Shell;
    if shell_l == "fish".to_string() {
        shell = Shell::Fish;
    } else if shell_l == "zsh".to_string() {
        shell = Shell::Zsh;
    } else if shell_l == "powershell".to_string() {
        shell = Shell::PowerShell;
    } else if shell_l == "elvish".to_string() {
        shell = Shell::Elvish;
    } else {
        shell = Shell::Bash;
    }

    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;

    let mut path = BufWriter::new(match args.value_of("out") {
        Some(x) => Box::new(
            File::create(&std::path::Path::new(x)).unwrap_or_else(|err| {
                eprintln!("Error opening file: {}", err);
                std::process::exit(1);
            }),
        ) as Box<Write>,
        None => Box::new(std::io::stdout()) as Box<Write>,
    });

    app.gen_completions_to("raspi_firmware", shell, &mut path);
}
