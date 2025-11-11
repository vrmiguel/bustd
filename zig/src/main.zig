const std = @import("std");

test "imports" {
    _ = @import("pressure.zig");
    _ = @import("daemonize.zig");
    _ = @import("process.zig");
    _ = @import("memory.zig");
    _ = @import("monitor.zig");
    _ = @import("config.zig");
}

const pressure = @import("pressure.zig");
const daemon = @import("daemonize.zig");
const process = @import("process.zig");
const memory = @import("memory.zig");
const monitor = @import("monitor.zig");
const config = @import("config.zig");
const syscalls = @import("missing_syscalls.zig");
const MCL = syscalls.MCL;

pub fn startMonitoring() anyerror!void {
    if (config.should_daemonize) {
        try daemon.daemonize();
    }

    var buffer: [128]u8 = undefined;

    if (syscalls.mlockall(MCL.CURRENT | MCL.FUTURE | MCL.ONFAULT)) {
        std.log.warn("Memory pages locked.", .{});
    } else |err| {
        std.log.warn("Failed to lock memory pages: {}. Continuing.", .{err});
    }

    var m = try monitor.Monitor.new(&buffer);
    try m.poll();
}

pub fn main() anyerror!void {
    startMonitoring() catch |err| {
        // If config.retry is set, get back up and running
        if (config.retry) {
            std.log.err("{s}. Continuing.", .{err});
            try main();
        } else {
            return err;
        }
    };
}
