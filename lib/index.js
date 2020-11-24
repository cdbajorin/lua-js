const lua54 = require("../native/lua54.node");
const lua53 = require("../native/lua53.node");
const lua52 = require("../native/lua52.node");
const lua51 = require("../native/lua51.node");
const luajit = require("../native/luajit.node");

exports.createLuaState = function (options) {
    const {version, ...stateOptions} = options;
    switch (version) {
        case "lua54": {
            return new lua54.LuaState(stateOptions);
        }
        case "lua53": {
            return new lua53.LuaState(stateOptions);
        }
        case "lua52": {
            return new lua52.LuaState(stateOptions);
        }
        case "lua51": {
            return new lua51.LuaState(stateOptions);
        }
        case "luajit": {
            return new luajit.LuaState(stateOptions);
        }
        default:
            throw new Error("Invalid Lua version");
    }
}

const SharedLibs = {
    table: 0x2,
    io: 0x4,
    os: 0x8,
    string: 0x10,
    math: 0x80,
    package: 0x100,
    debug: 0x80000000,
    ALL: 0xFFFFFFFF,
    ALL_SAFE: 0xFFFFFFFE,
}
exports.Lua51 = {
    Version: "lua51",
    Libs: Object.assign({}, SharedLibs)
}

exports.Lua52 = {
    Version: "lua52",
    Libs: Object.assign({bit: 0x40}, SharedLibs)
}

exports.Lua53 = {
    Version: "lua53",
    Libs: Object.assign({coroutine: 0x1, utf8: 0x20}, SharedLibs)
}

exports.Lua54 = {
    Version: "lua54",
    Libs: Object.assign({coroutine: 0x1, utf8: 0x20}, SharedLibs)
}

exports.LuaJIT = {
    Version: "luajit",
    Libs: Object.assign({bit: 0x40, jit: 0x200, ffi: 0x40000000}, SharedLibs)
}