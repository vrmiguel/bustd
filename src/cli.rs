use argh;
use argh::FromArgs;
// use argh::;

#[derive(FromArgs)]
/// Lightweight process killer daemon for out-of-memory scenarios
pub struct CommandLineArgs {
    /// when set, bustd will kill the victim's entire process group
    #[argh(switch, short = 'g')]
    pub group: bool,

    /// when set, the process will not be daemonized
    #[argh(switch, short = 'n')]
    pub no_daemon: bool,

    /// when set, the victim's entire process group will be killed
    #[argh(switch, short = 'g')]
    pub kill_pgroup: bool,

    /// sets the PSI value on which, if surpassed, a process will be killed
    #[argh(option, short = 'p', long = "psi", default = "25.0")]
    pub cutoff_psi: f32, // TODO: responsitivity multiplier?
}
