# `bustd`: Available memory or bust!

`bustd` is a lightweight process killer daemon for out-of-memory scenarios for Linux!

## Features

### Small memory usage!

`bustd` seems to use less memory than some other lean daemons such as `earlyoom`:

```console
$ ps -F -C bustd
UID          PID    PPID  C    SZ   RSS PSR STIME TTY          TIME CMD
vrmiguel  353609  187407  5   151     8   2 01:20 pts/2    00:00:00 target/x86_64-unknown-linux-musl/release/bustd -V -n

$ ps -F -C earlyoom
UID          PID    PPID  C    SZ   RSS PSR STIME TTY          TIME CMD
vrmiguel  350497    9498  0   597   688   6 01:12 pts/1    00:00:00 ./earlyoom/
```

¹: RSS stands for resident set size and represents the portion of RAM occupied by a process.

²: Compared when bustd was in [this commit](https://github.com/vrmiguel/bustd/commit/61beb097b3631afb231a76bb9187b802c9818793) and earlyoom in [this one](https://github.com/rfjakob/earlyoom/commit/509df072be79b3be2a1de6581499e360ab0180be).
`bustd` compiled with musl libc and earlyoom with glibc through GCC 11.1. Different configurations would likely change these figures.


### Small CPU usage

Much like `earlyoom` and `nohang`, `bustd` uses adaptive sleep times during its memory polling. Unlike these two, however, `bustd` does not read from `/proc/meminfo`, instead opting for the `sysinfo` syscall.

This approach has its up- and downsides. The amount of free RAM that `sysinfo` reads does not account for cached memory, while `MemAvailable` in `/proc/meminfo` does.

The `sysinfo` syscall is one order of magnitude faster, at least according to [this kernel patch](https://sourceware.org/legacy-ml/libc-alpha/2015-08/msg00512.html) (granted, from 2015).

As `bustd` can't solely rely on the free RAM readings of `sysinfo`, we check for memory stress through [Pressure Stall Information](https://www.kernel.org/doc/html/v5.8/accounting/psi.html).

### `bustd` will try to lock all pages mapped into its address space

Much like `earlyoom`, `bustd` uses [`mlockall`](https://www.ibm.com/docs/en/aix/7.2?topic=m-mlockall-munlockall-subroutine) to avoid being sent to swap, which allows the daemon to remain responsive even when the system memory is under heavy load and susceptible to [thrashing](https://en.wikipedia.org/wiki/Thrashing_(computer_science)).

### Checks for Pressure Stall Information

The Linux kernel, since version 4.20 (and built with `CONFIG_PSI=y`), presents canonical new pressure metrics for memory, CPU, and IO.
In the words of [Facebook Incubator](https://facebookmicrosites.github.io/psi/docs/overview):

```
PSI stats are like barometers that provide fair warning of impending resource 
shortages, enabling you to take more proactive, granular, and nuanced steps 
when resources start becoming scarce.
```

More specifically, `bustd` checks for how long, in microseconds, processes have stalled in the last 10 seconds. By default, `bustd` will kill a process when processes have stalled for 25 microseconds in the last ten seconds.

## Building

Requirements:
* [Rust toolchain](https://rustup.rs/)
* Any C compiler
* Linux 4.20+ built with `CONFIG_PSI=y`

```shell
git clone https://github.com/vrmiguel/bustd
cd bustd && cargo run --release
```

The `-n, --no-daemon` flag is useful for running `bustd` through an init system such as `systemd`.

## Prebuilt binaries

Binaries are generated at every commit through [GitHub Actions](https://github.com/vrmiguel/bustd/actions)

## TODO

- [x] Allow for customization of the critical scenario (PSI cutoff)
- [x] Command-line argument for disabling daemonization (useful for runnning `bustd` as a systemd service)
- [x] Command-line argument to enable killing the entire process group, not just the chosen process itself
- [ ] Notification sending and general notification customization settings
- [ ] Allow the user to setup a list of software that `bustd` should never kill
