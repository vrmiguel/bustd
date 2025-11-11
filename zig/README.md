:warning: This will be marged with the original [bustd](https://github.com/vrmiguel/bustd) repository.

# `buztd`: Available memory or bust!

`buztd` is a lightweight process killer daemon for out-of-memory scenarios for Linux!

This particular project is a Zig version of the [original `bustd` project](https://github.com/vrmiguel/bustd).

## Features

### Extremely thin memory usage

The Zig version of `bustd` makes no heap allocations and relies solely on a single 128-byte buffer in the stack for all its allocation needs. 

### Small CPU usage

Much like `earlyoom` and `nohang`, `buztd` uses adaptive sleep times during its memory polling. 

Unlike these two, however, `buztd` does not read from `/proc/meminfo`, instead opting for the `sysinfo` syscall.

This approach has its up- and downsides. The amount of free RAM that `sysinfo` reads does not account for cached memory, while `MemAvailable` in `/proc/meminfo` does.

However, the `sysinfo` syscall is one order of magnitude faster than parsing `/proc/meminfo`, at least according to [this kernel patch](https://sourceware.org/legacy-ml/libc-alpha/2015-08/msg00512.html) (granted, from 2015).

As `buztd` can't solely rely on the free RAM readings of `sysinfo`, we check for memory stress through [Pressure Stall Information](https://www.kernel.org/doc/html/v5.8/accounting/psi.html).

More on that below.

### `buztd` will try to lock all pages mapped into its address space

Much like `earlyoom`, `buztd` uses [`mlockall`](https://www.ibm.com/docs/en/aix/7.2?topic=m-mlockall-munlockall-subroutine) to avoid being sent to swap, which allows the daemon to remain responsive even when the system memory is under heavy load and susceptible to [thrashing](https://en.wikipedia.org/wiki/Thrashing_(computer_science)).

### Checks for Pressure Stall Information

The Linux kernel, since version 4.20 (and built with `CONFIG_PSI=y`), presents canonical new pressure metrics for memory, CPU, and IO.
In the words of [Facebook Incubator](https://facebookmicrosites.github.io/psi/docs/overview):

```
PSI stats are like barometers that provide fair warning of impending resource 
shortages, enabling you to take more proactive, granular, and nuanced steps 
when resources start becoming scarce.
```

More specifically, `buztd` checks for how long, in microseconds, processes have stalled in the last 10 seconds. By default, `buztd` will kill a process when processes have stalled for 25 microseconds in the last ten seconds.

Example:
```
   some avg10=0.00 avg60=0.00 avg300=0.00 total=11220657
   full avg10=0.00 avg60=0.00 avg300=0.00 total=10947429
```

These ratios are percentages of recent trends over ten, sixty, and  three hundred second windows.

The `some` row indicates the percentage of time n that given time frame in which _any_ process has stalled due to memory thrashing.

`buztd` allows you to configure the value of `some avg10` in which, if surpassed, some process will be killed.

The ideal value for this cutoff varies a lot between systems.

Try messing around with `tools/mem-eater.c` to guesstimate a value that works well for you.

## Building

Requirements:
* [Zig 0.10](https://ziglang.org/)
* Linux 4.20+ built with `CONFIG_PSI=y`

```shell
git clone https://github.com/vrmiguel/buztd
cd buztd

# Choose which compilation mode you'd like:
zig build -Drelease-fast # Turns on optimization and disables safety checks
zig build -Drelease-safe # Turns on optimization and keeps safety checks
zig build -Drelease-small # Turns on size optimizations and disables safety checks
```

## Configuration

As of the time of writing, this version of `buztd` offers no command-line argument parsing, but allows easy configuration through the `src/config.zig` file.


```zig
/// Sets whether or not buztd should daemonize
/// itself. Don't use this if running buztd as a systemd
/// service or something of the sort.
pub const should_daemonize: bool = false;

/// Free RAM percentage figures below this threshold are considered to be near terminal, meaning 
/// that buztd will start to check for Pressure Stall Information whenever the
/// free RAM figures go below this.
/// However, this free RAM amount is what the sysinfo syscall gives us, which does not take in consideration
/// reclaimable or cached pages. The true free RAM amount available to the OS is bigger than what it indicates.
pub const free_ram_threshold: u8 = 15;

/// The Linux kernel presents canonical pressure metrics for memory, found in `/proc/pressure/memory`.
/// Example:
///    some avg10=0.00 avg60=0.00 avg300=0.00 total=11220657
///    full avg10=0.00 avg60=0.00 avg300=0.00 total=10947429
/// These ratios are percentages of recent trends over ten, sixty, and 
/// three hundred second windows. The `some` row indicates the percentage of time
// in that given time frame in which _any_ process has stalled due to memory thrashing.
///
/// This value configured here is the value of `some avg10` in which, if surpassed, some 
/// process will be killed.
///
/// The ideal value for this cutoff varies a lot between systems.
/// Try messing around with `tools/mem-eater.c` to guesstimate a value that works well for you.
pub const cutoff_psi: f32 = 0.05;

/// Sets processes that buztd must never kill.
/// The values expected here are the `comm` values of the process you don't want to have terminated.
/// A comm-value is the filename of the executable truncated to 16 characters..
pub const unkillables = std.ComptimeStringMap(void, .{
         .{ "firefox", void },
         .{ "rustc", void },
         .{ "electron", void },
});


/// If any error occurs, restarts the monitoring instead of exiting with an unsuccesful status code
pub const retry: bool = true;
```


