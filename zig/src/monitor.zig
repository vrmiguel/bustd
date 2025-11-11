const std = @import("std");
const time = std.time;
const math = std.math;

const memory = @import("memory.zig");
const pressure = @import("pressure.zig");
const process = @import("process.zig");
const config = @import("config.zig");

const MemoryStatusTag = enum {
    ok,
    near_terminal,
};

const MemoryStatus = union(MemoryStatusTag) {
    /// Memory is "okay": basically no risk of memory thrashing
    ok: void,
    /// Nearing the terminal PSI cutoff: memory thrashing is occurring or close to it. Holds the current PSI value.
    near_terminal: f32,
};

pub const Monitor = struct {
    mem_info: memory.MemoryInfo,
    /// Memory status as of last checked
    status: MemoryStatus,
    /// A pointer to a buffer of at least 128 bytes
    buffer: []u8,
    const Self = @This();

    pub fn new(buffer: []u8) !Self {
        var self = Self{
            .mem_info = undefined,
            .status = undefined,
            .buffer = buffer,
        };

        try self.updateMemoryStats();

        return self;
    }

    pub fn updateMemoryStats(self: *Self) !void {
        self.mem_info = try memory.MemoryInfo.new();
        self.status = blk: {
            if (self.mem_info.available_ram_percent <= config.free_ram_threshold) {
                const psi = try pressure.pressureSomeAvg10(self.buffer);
                std.log.warn("read avg10: {}", .{psi});
                break :blk MemoryStatus{ .near_terminal = psi };
            } else {
                break :blk MemoryStatus.ok;
            }
        };
    }

    fn freeUpMemory(self: *Self) !void {
        const victim_process = try process.findVictimProcess(self.buffer);

        // Check for memory stats again to see if the
        // low-memory situation was solved while
        // we were searching for our victim
        try self.updateMemoryStats();
        if (self.isMemoryLow()) {
            try victim_process.terminateSelf();
        }
    }

    pub fn poll(self: *Self) !void {
        while (true) {
            if (self.isMemoryLow()) {
                try self.freeUpMemory();
            }

            try self.updateMemoryStats();
            const sleep_time = self.sleepTimeNs();
            std.log.warn("sleeping for {}ms, {}% of RAM is free", .{ sleep_time, self.mem_info.available_ram_percent });

            // Convert ms to ns
            time.sleep(sleep_time * 1000000);
        }
    }

    /// Determines for how long buztd should sleep
    /// This function is essentially a copy of how earlyoom calculates its sleep time
    ///
    /// Credits: https://github.com/rfjakob/earlyoom/blob/dea92ae67997fcb1a0664489c13d49d09d472d40/main.c#L365
    /// MIT Licensed
    fn sleepTimeNs(self: *const Self) u64 {
        // Maximum expected memory fill rate as seen
        // with `stress -m 4 --vm-bytes 4G`
        const ram_fill_rate: i64 = 6000;
        // Maximum expected swap fill rate as seen
        // with membomb on zRAM
        const swap_fill_rate: i64 = 800;

        // Maximum and minimum sleep times (in ms)
        const min_sleep: i64 = 100;
        const max_sleep: i64 = 1000;

        // TODO: make these percentages configurable by args./config. file
        const ram_terminal_percent: f64 = 10.0;
        const swap_terminal_percent: f64 = 10.0;

        const f_ram_headroom_kib = (@intToFloat(f64, self.mem_info.available_ram_percent) - ram_terminal_percent) * (@intToFloat(f64, self.mem_info.total_ram_mb) * 10.0);
        const f_swap_headroom_kib = (@intToFloat(f64, self.mem_info.available_swap_percent) - swap_terminal_percent) * (@intToFloat(f64, self.mem_info.total_swap_mb) * 10.0);

        const i_ram_headroom_kib = math.max(0, @floatToInt(i64, f_ram_headroom_kib));
        const i_swap_headroom_kib = math.max(0, @floatToInt(i64, f_swap_headroom_kib));

        var time_to_sleep = @divFloor(i_ram_headroom_kib, ram_fill_rate) + @divFloor(i_swap_headroom_kib, swap_fill_rate);
        time_to_sleep = math.min(time_to_sleep, max_sleep);
        time_to_sleep = math.max(time_to_sleep, min_sleep);

        return @intCast(u64, time_to_sleep);
    }

    fn isMemoryLow(self: *const Self) bool {
        return switch (self.status) {
            MemoryStatusTag.ok => false,
            MemoryStatusTag.near_terminal => |psi| psi >= config.cutoff_psi,
        };
    }
};
