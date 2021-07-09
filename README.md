# `bustd`: Available memory or bust!

`bustd` is a lightweight process killer daemon for out-of-memory scenarios for Linux!

## Features

### Insignificant memory usage!

`bustd` seems to use less memory than some other lean daemons such as `earlyoom`:

```console
$ ps -F -C bust
UID          PID    PPID  C    SZ   RSS PSR STIME TTY          TIME CMD
vrmiguel   42470       1  0   152    60   4 jul08 ?        00:00:00 ./target/x86_64-unknown-linux-musl/release/bust
$ ps -F -C earlyoom
UID          PID    PPID  C    SZ   RSS PSR STIME TTY          TIME CMD
vrmiguel   42705    2351  0   597   680   3 jul08 pts/1    00:00:00 /home/vrmiguel/earlyoom/earlyoom
```

¹: RSS stands for resident set size and represents the portion of RAM occupied by a process.

²: Compared when bustd was in [this commit](https://github.com/vrmiguel/bust/commit/c19723762815620a7e05b2a829a462e650656488) and earlyoom in [this one](https://github.com/rfjakob/earlyoom/commit/509df072be79b3be2a1de6581499e360ab0180be).
`bustd` compiled with musl libc and earlyoom with glibc through GCC 11.1. Different configurations might change these figures.


### Also quite insignificant CPU usage

Much like `earlyoom` and `nohang`, `bustd` uses adaptive sleep times during its memory polling. Unlike these two, however, `bustd` does not read from `/proc/meminfo`, instead opting for the `sysinfo` syscall.

This approach has its up- and downsides. The amount of free RAM that `sysinfo` reads does not account for cached memory, while `MemAvailable` in `/proc/meminfo` does.

The `sysinfo` syscall is one order of magnitude faster, at least according to [this kernel patch](https://sourceware.org/legacy-ml/libc-alpha/2015-08/msg00512.html) (granted, from 2015).

As `bustd` can't solely rely on the free RAM readings of `sysinfo`, we check for memory stress through [Pressure Stall Information](https://www.kernel.org/doc/html/v5.8/accounting/psi.html).



## TODO

`bustd` is still in its infancy

- [ ] Allow for customization of the critical scenario
- [ ] Command-line argument for disabling daemonization (useful for runnning `bustd` as a systemd service)
- [ ] Command-line argument to enable killing the entire process group, not just the chosen process itself
- [ ] Notification sending and general notification customization settings
- [ ] Allow the user to setup a list of software that `bustd` should never kill
