// Until mlockall and sysinfo are added to zig's stdlib

const std = @import("std");
const os = std.os;
const l = std.os.linux;

// Flag magic numbers
pub const MCL = struct {
    pub const CURRENT = 1;
    pub const FUTURE = 2;
    pub const ONFAULT = 4;
};

// TODO: test if this works outside of x86_64
/// Contains certain statistics on memory and swap usage, as well as the load average
pub const SysInfo = extern struct {
    uptime: c_long,
    loads: [3]c_ulong,
    totalram: c_ulong,
    freeram: c_ulong,
    sharedram: c_ulong,
    bufferram: c_ulong,
    totalswap: c_ulong,
    freeswap: c_ulong,
    procs: c_ushort,
    totalhigh: c_ulong,
    freehigh: c_ulong,
    mem_unit: c_int,
    // pad
    _f: [20 - 2 * @sizeOf(c_long) - @sizeOf(c_int)]u8,
};

pub const MLockError = error{ CouldNotLock, SystemResources, PermissionDenied } || os.UnexpectedError;

fn syscall_mlockall(flags: i32) usize {
    return l.syscall1(.mlockall, @bitCast(usize, @as(isize, flags)));
}

pub fn mlockall(flags: i32) MLockError!void {
    const rc = l.getErrno(syscall_mlockall(flags));
    switch (rc) {
        .SUCCESS => return,
        .AGAIN => return error.CouldNotLock,
        .PERM => return error.PermissionDenied,
        .NOMEM => return error.SystemResources,
        .INVAL => unreachable,
        else => |err| return os.unexpectedErrno(err),
    }
}

fn syscall_sysinfo(info: *SysInfo) usize {
    return l.syscall1(.sysinfo, @ptrToInt(info));
}

pub fn sysinfo(info: *SysInfo) os.UnexpectedError!void {
    const rc = l.getErrno(syscall_sysinfo(info));
    switch (rc) {
        .SUCCESS => return,
        .FAULT => unreachable,
        else => |err| return os.unexpectedErrno(err),
    }
}
