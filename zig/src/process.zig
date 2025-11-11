const std = @import("std");
const csig = @cImport({
    @cInclude("signal.h");
});
const unistd = @cImport({
    @cInclude("unistd.h");
});
const config = @import("config.zig");
const fs = std.fs;
const fmt = std.fmt;
const mem = std.mem;
const os = std.os;
const libc = std.c;
const time = std.time;

fn signalToString(signal: u8) []const u8 {
    return switch (signal) {
        csig.SIGTERM => "SIGTERM",
        csig.SIGKILL => "SIGKILL",
        else => "unknown",
    };
}

pub const Process = struct {
    pid: u32,
    oom_score: i16,
    buffer: []u8,

    const Self = @This();

    const ProcessError = error{ MalformedOomScore, MalformedOomScoreAdj, MalformedVmRss };

    fn fromPid(pid: u32, buffer: []u8) !Self {
        const oom_score = try oomScoreFromPid(pid, buffer);

        return Self{ .pid = pid, .oom_score = oom_score, .buffer = buffer };
    }

    fn oomScoreFromPid(pid: u32, buffer: []u8) !i16 {
        const path = try fmt.bufPrint(buffer, "/proc/{}/oom_score", .{pid});

        // The file descriptor for the oom_score file of this process
        const oom_score_fd = try os.open(path, os.O.RDONLY, 0);
        defer os.close(oom_score_fd);

        const bytes_read = try os.read(oom_score_fd, buffer);

        const oom_score = parse(i16, buffer[0 .. bytes_read - 1]) orelse return error.MalformedOomScore;

        return oom_score;
    }

    pub fn oomScoreAdj(self: *const Self) !i16 {
        const path = try fmt.bufPrint(self.buffer, "/proc/{}/oom_score_adj", .{self.pid});

        // The file descriptor for the oom_score file of this process
        const oom_score_adj_fd = try os.open(path, os.O.RDONLY, 0);
        defer os.close(oom_score_adj_fd);

        const bytes_read = try os.read(oom_score_adj_fd, self.buffer);

        const oom_score_adj = parse(i16, self.buffer[0 .. bytes_read - 1]) orelse return error.MalformedOomScoreAdj;

        return oom_score_adj;
    }

    pub fn comm(self: *const Self) ![]u8 {
        const path = try fmt.bufPrint(self.buffer, "/proc/{}/comm", .{self.pid});

        // The file descriptor for the oom_score file of this process
        const comm_fd = try os.open(path, os.O.RDONLY, 0);
        defer os.close(comm_fd);

        const bytes_read = try os.read(comm_fd, self.buffer);

        return self.buffer[0 .. bytes_read - 1];
    }

    pub fn isAlive(self: *const Self) bool {
        const group_id = unistd.getpgid(@intCast(c_int, self.pid));

        return group_id > 0;
    }

    pub fn vmRss(self: *const Self) !usize {
        var filename = try fmt.bufPrint(self.buffer, "/proc/{}/statm", .{self.pid});

        var statm_file = try fs.cwd().openFile(filename, .{});
        defer statm_file.close();
        var statm_reader = statm_file.reader();

        // Skip first field (total program size)
        try statm_reader.skipUntilDelimiterOrEof(' ');
        var rss_str = try statm_reader.readUntilDelimiter(self.buffer, ' ');

        var ret = parse(usize, rss_str) orelse return error.MalformedVmRss;
        return (ret * std.mem.page_size) / 1024;
    }

    pub fn signalSelf(self: *const Self, signal: u8) void {
        // Don't warn `kill` failure if the process is no longer alive
        if (0 != libc.kill(@intCast(i32, self.pid), signal) and self.isAlive()) {
            std.log.warn("Failed to send {s} to process {}", .{ signalToString(signal), self.pid });
        } else {
            std.log.warn("Successfully sent {s} to process {}", .{ signalToString(signal), self.pid });
        }
    }

    pub fn terminateSelf(self: Self) !void {
        const quarter_sec_in_ns: u64 = 250000000;

        self.signalSelf(csig.SIGTERM);

        var attempt: u8 = 0;

        while (attempt < 20) : (attempt += 1) {
            time.sleep(quarter_sec_in_ns);
            if (!self.isAlive()) {
                std.log.warn("Process {} has exited.", .{self.pid});
                return;
            }
            // Escalate to sigkill
            self.signalSelf(csig.SIGKILL);
        }
    }
};

/// Wrapper over fmt.parseInt which returns null
/// in failure instead of an error
fn parse(comptime T: type, buf: []const u8) ?T {
    return fmt.parseInt(T, buf, 10) catch null;
}

/// Used to try to give LLVM hints on branch prediction.
///
/// I have no idea how effective this is in practice.
fn coldNoOp() void {
    @setCold(true);
}

/// Searches for a process to kill in order to
/// free up memory
pub fn findVictimProcess(buffer: []u8) !Process {
    var victim: Process = undefined;
    var victim_vm_rss: usize = undefined;
    var victim_is_undefined = true;

    var timer = try time.Timer.start();

    var proc_dir = try fs.cwd().openIterableDir("/proc", .{ .access_sub_paths = false });
    var proc_it = proc_dir.iterate();

    while (try proc_it.next()) |proc_entry| {
        // We're only interested in directories of /proc
        if (proc_entry.kind != .Directory) {
            continue;
        } else {
            // `/proc` usually has much more directories than it has files
            coldNoOp();
        }

        // But we're not interested in files that don't relate to a PID
        const pid = parse(u32, proc_entry.name) orelse continue;

        // Don't consider killing the init system
        if (pid <= 1) {
            coldNoOp();
            continue;
        }

        const process = try Process.fromPid(pid, buffer);

        if (victim_is_undefined) {
            // We're still reading the first process so a victim hasn't been chosen
            coldNoOp();
            victim = process;
            victim_vm_rss = try victim.vmRss();
            victim_is_undefined = false;
            std.log.info("First victim set", .{});
        }

        if (victim.oom_score > process.oom_score) {
            // Our current victim is less innocent than the process being analysed
            continue;
        }

        const victim_comm = try victim.comm();
        if (config.unkillables.get(victim_comm) != null) {
            // The current process was set as unkillable
            continue;
        }

        const current_vm_rss = try process.vmRss();
        if (current_vm_rss == 0) {
            // Current process is a kernel thread
            continue;
        }

        // TODO: recheck this
        if (process.oom_score == victim.oom_score and current_vm_rss <= victim_vm_rss) {
            continue;
        }

        const current_oom_score_adj = process.oomScoreAdj() catch {
            std.log.warn("Failed to read adj. OOM score for PID {}. Continuing.", .{process.pid});
            continue;
        };

        if (current_oom_score_adj == -1000) {
            // Follow the behaviour of the standard OOM killer: don't kill processes with oom_score_adj equals to -1000
            continue;
        }

        victim = process;
        victim_vm_rss = current_vm_rss;
    }

    const ns_elapsed = timer.read();
    std.debug.print("Victim found in {} ns.: {s} with PID {} and OOM score {}\n", .{ ns_elapsed, try victim.comm(), victim.pid, victim.oom_score });

    return victim;
}
