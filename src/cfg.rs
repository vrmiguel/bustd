use clap::{App, Arg};

pub struct Config {
    /// toggles on verbose output
    pub verbose: bool,
    /// when set, the process will not be daemonized
    pub daemonize: bool,
    /// when set, the victim's entire process group will be killed
    pub kill_pgroup: bool,
    /// sets the PSI value on which, if surpassed, a process will be killed
    pub cutoff_psi: f32,
    // #[cfg(feature = "glob-ignore")]
    /// all processes whose names match any of the supplied tilde-separated glob patterns will never be chosen to be killed
    pub ignored_globs: Option<Vec<String>>,
}

fn make_app<'a, 'b>() -> App<'a, 'b> {
    App::new("bustd")
            .version("0.1.0")
            .author("vrmiguel")
            .about("Lightweight Process-Killer Daemon for Out-of-Memory Scenarios")
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .help("Log extra debugging information"),
            )
            .arg(
                Arg::with_name("daemonize")
                    .short("n")
                    .long("no-daemon")
                    .help("Do not fork a daemon process"),
            )
            .arg(
                Arg::with_name("pgroup")
                    .short("g")
                    .long("pgroup")
                    .help("Kill the entire process group of a target"),
            )
            .arg(
                Arg::with_name("cutoff")
                    .short("p")
                    .long("psi")
                    .help("The cieling PSI value; All processes that surpass this will be killed")
                    .default_value("25.0"),
            )
            .arg(
                Arg::with_name("ignored")
                    .short("u")
                    .long("unkillable")
                    .help(
                    "A (tilde-separated) list of glob patterns matching process names which will never be killed")
                    .value_delimiter("~"),
            )
}

impl Config {
    pub fn parse_args() -> Self {
        let matches = make_app().get_matches();
        Self {
            verbose: matches.is_present("verbose"),
            daemonize: !matches.is_present("daemonize"),
            kill_pgroup: matches.is_present("pgroup"),
            cutoff_psi: matches.value_of("cutoff").unwrap().parse().unwrap(),
            ignored_globs: matches
                .values_of("ignored")
                .and_then(|globs| Some(globs.map(String::from).collect())),
        }
    }
}
