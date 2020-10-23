const addon = require("../native");
exports.LuaState = addon.LuaState;

// bitflags corresponding to rlua libraries
// https://github.com/amethyst/rlua/blob/229743776e8bbf25a2e486e6e54971af1e21fc73/src/lua.rs#L27-L36
const LuaLib = {
    Base: 1,
    Coroutine: 1 << 1,
    Table: 1 << 2,
    Io: 1 << 3,
    Os: 1 << 4,
    String: 1 << 5,
    Utf8: 1 << 6,
    Math: 1 << 7,
    Package: 1 << 8,
    Debug: 1 << 9,
}
exports.LuaLibs = LuaLib;

const LuaLibSet = {
    All: [LuaLib.Base, LuaLib.Coroutine, LuaLib.Table, LuaLib.Io, LuaLib.Os, LuaLib.String, LuaLib.Utf8, LuaLib.Math, LuaLib.Package, LuaLib.Debug],
    AllNoDebug: [LuaLib.Base, LuaLib.Coroutine, LuaLib.Table, LuaLib.Io, LuaLib.Os, LuaLib.String, LuaLib.Utf8, LuaLib.Math, LuaLib.Package],
    NoFs: [LuaLib.Base, LuaLib.Coroutine, LuaLib.Table, LuaLib.String, LuaLib.Utf8, LuaLib.Math, LuaLib.Package],
}
exports.LuaLibSet = LuaLibSet;
