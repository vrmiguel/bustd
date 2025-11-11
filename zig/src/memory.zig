const syscalls = @import("missing_syscalls.zig");

pub const MemoryInfo = struct {
    const Self = @This();

    total_ram_mb: u64,
    total_swap_mb: u64,
    available_ram_mb: u64,
    available_swap_mb: u64,
    available_ram_percent: u8,
    available_swap_percent: u8,

    fn bytes_to_megabytes(bytes: u64, mem_unit: u64) u64 {
        const B_TO_MB: u64 = 1000 * 1000;
        return bytes / B_TO_MB * mem_unit;
    }

    fn ratio(x: u64, y: u64) u8 {
        const xf = @intToFloat(f32, x);
        const yf = @intToFloat(f32, y);

        const _ratio = (xf / yf) * 100.0;

        return @floatToInt(u8, _ratio);
    }

    pub fn new() !Self {
        var si: syscalls.SysInfo = undefined;
        try syscalls.sysinfo(&si);

        const mem_unit = @intCast(u64, si.mem_unit);

        const available_ram_mb = bytes_to_megabytes(si.freeram, mem_unit);
        const total_ram_mb = bytes_to_megabytes(si.totalram, mem_unit);
        const total_swap_mb = bytes_to_megabytes(si.totalswap, mem_unit);
        const available_swap_mb = bytes_to_megabytes(si.freeswap, mem_unit);
        const available_ram_percent = ratio(available_ram_mb, total_ram_mb);
        const available_swap_percent = blk: {
            if (total_swap_mb != 0) {
                break :blk ratio(available_swap_mb, total_swap_mb);
            } else {
                break :blk 0;
            }
        };

        return Self{
            .available_ram_mb = available_ram_mb,
            .total_ram_mb = total_ram_mb,
            .total_swap_mb = total_swap_mb,
            .available_swap_mb = available_swap_mb,
            .available_ram_percent = available_ram_percent,
            .available_swap_percent = available_swap_percent,
        };
    }
};
