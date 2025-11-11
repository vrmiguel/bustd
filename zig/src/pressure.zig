const std = @import("std");
const os = std.os;
const fmt = std.fmt;
const fs = std.fs;
const mem = std.mem;
const assert = std.debug.assert;

pub fn pressureSomeAvg10(buffer: []u8) !f32 {
    assert(buffer.len >= 128);

    const memory_pressure_file = try fs.cwd().openFile("/proc/pressure/memory", .{});
    defer memory_pressure_file.close();

    var memory_pressure_reader = memory_pressure_file.reader();

    // Read "some"
    const some = try memory_pressure_reader.readUntilDelimiter(buffer, ' ');
    assert(mem.eql(u8, some, "some"));

    // Read "avg10=" (`readUntilDelimiter` will eat the '=')
    const avg10 = try memory_pressure_reader.readUntilDelimiter(buffer, '=');
    assert(mem.eql(u8, avg10, "avg10"));

    // Next up is the value we want
    const avg10_value = try memory_pressure_reader.readUntilDelimiter(buffer, ' ');
    std.log.info("avg10: {s}", .{avg10_value});

    return try fmt.parseFloat(f32, avg10_value);
}
