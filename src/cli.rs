use argh::FromArgs;

#[derive(FromArgs)]
/// Lightweight process killer daemon for out-of-memory scenarios
pub struct CommandLineArgs {
    /// toggles on verbose output
    #[argh(switch, short = 'V')]
    pub verbose: bool,

    /// when set, the process will not be daemonized
    #[argh(switch, short = 'n')]
    pub no_daemon: bool,

    /// when set, the victim's entire process group will be killed
    #[argh(switch, short = 'g')]
    pub kill_pgroup: bool,

    /// sets the PSI value on which, if surpassed, a process will be killed
    #[argh(option, short = 'p', long = "psi", default = "25.0")]
    pub cutoff_psi: f32, // TODO: responsitivity multiplier?

    #[cfg(feature = "glob-ignore")]
    /// all processes whose names match any of the supplied tilde-separated glob patterns will never be chosen to be killed
    #[argh(
        option,
        short = 'u',
        long = "unkillables",
        from_str_fn(parse_unkillables)
    )]
    pub ignored: Option<Vec<String>>,
}

#[cfg(feature = "glob-ignore")]
fn parse_unkillables(arg: &str) -> Result<Vec<String>, String> {
    Ok(arg.split('~').map(ToOwned::to_owned).collect())
}
