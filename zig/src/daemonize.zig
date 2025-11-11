const std = @import("std");
const os = std.os;

const unistd = @cImport({
    @cInclude("unistd.h");
});

const signal = @cImport({
    @cInclude("signal.h");
});

const stat = @cImport({
    @cInclude("sys/stat.h");
});

/// Any error that might come up during process daemonization
const DaemonizeError = error{FailedToSetSessionId} || os.ForkError;

const SignalHandler = struct {
    fn ignore(sig: i32, info: *const os.siginfo_t, ctx_ptr: ?*const anyopaque) callconv(.C) void {
        // Ignore the signal received
        _ = sig;
        _ = ctx_ptr;
        _ = info;
        _ = ctx_ptr;
    }
};

/// Forks the current process and makes
/// the parent process quit
fn fork_and_keep_child() os.ForkError!void {
    const is_parent_proc = (try os.fork()) != 0;
    // Exit off of the parent process
    if (is_parent_proc) {
        os.exit(0);
    }
}

// TODO:
// * Add logging
// * Chdir
/// Daemonizes the calling process
pub fn daemonize() DaemonizeError!void {
    try fork_and_keep_child();

    if (unistd.setsid() < 0) {
        return error.FailedToSetSessionId;
    }

    // Setup signal handling
    var act = os.Sigaction{
        .handler = .{ .sigaction = SignalHandler.ignore },
        .mask = os.empty_sigset,
        .flags = (os.SA.SIGINFO | os.SA.RESTART | os.SA.RESETHAND),
    };
    os.sigaction(signal.SIGCHLD, &act, null);
    os.sigaction(signal.SIGHUP, &act, null);

    // Fork yet again and keep only the child process
    try fork_and_keep_child();

    // Set new file permissions
    _ = stat.umask(0);

    var fd: u8 = 0;
    // The maximum number of files a process can have open
    // at any time
    const max_files_opened = unistd.sysconf(unistd._SC_OPEN_MAX);
    while (fd < max_files_opened) : (fd += 1) {
        _ = unistd.close(fd);
    }
}

test "fork_and_keep_child works" {
    const getpid = os.linux.getpid;
    const expect = std.testing.expect;
    const linux = std.os.linux;
    const fmt = std.fmt;

    const first_pid = getpid();
    try fork_and_keep_child();

    const new_pid = getpid();
    // We should now be running on a new process
    try expect(first_pid != new_pid);

    var stat_buf: linux.Stat = undefined;
    var buf = [_:0]u8{0} ** 128;

    // Current process is alive (obviously)
    _ = try fmt.bufPrint(&buf, "/proc/{}/stat", .{new_pid});

    try expect(linux.stat(&buf, &stat_buf) == 0);

    // Old process should now be dead
    _ = try fmt.bufPrint(&buf, "/proc/{}/stat", .{first_pid});

    // Give the OS some time to reap the old process
    std.time.sleep(250_000);

    try expect(
    // Stat should now fail
    linux.stat(&buf, &stat_buf) != 0);
}
