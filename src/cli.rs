use argh::FromArgs;
use argh;
// use argh::;


#[derive(FromArgs)]
/// Lightweight process killer daemon for out-of-memory scenarios 
pub struct CommandLineArgs {
    /// when set, bustd will kill the victim's entire process group
    #[argh(switch, short = 'g')]
    group: bool,

    /// sets the PSI value on which, if surpassed, a process will be killed
    #[argh(option, short = 'p', long = "psi", default = "25.0")]
    cutoff_psi: f32

    // TODO: responsitivity multiplier?
}