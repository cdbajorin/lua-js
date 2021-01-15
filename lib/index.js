const lua54 = require("../native/lua54");
const lua53 = require("../native/lua53");
const lua52 = require("../native/lua52");
const lua51 = require("../native/lua51");
const luajit = require("../native/luajit");

const Symbols = {
    ZeroIndex: Symbol.for("LuaJs.0")
}
module.exports.Symbols = Symbols;

const getNativeModule = (version) => {
    switch (version) {
        case "lua54": {
            return lua54;
        }
        case "lua53": {
            return lua53;
        }
        case "lua52": {
            return lua52;
        }
        case "lua51": {
            return lua51;
        }
        case "luajit": {
            return luajit;
        }
        default:
            throw new Error("Invalid Lua version");
    }
}

function createLuaState(options) {
    const {version, ...stateOptions} = options;
    const nativeModule = getNativeModule(version);

    return new class {
        [Symbol.toStringTag] = `LuaState: ${version}`;

        constructor(options = {}) {
            this.native = nativeModule.LuaState_Constructor(options);
        }

        doStringSync(code, name = "?") {
            return nativeModule.LuaState_doStringSync(this.native, code, name);
        }

        doFileSync(filepath, name) {
            return nativeModule.LuaState_doFileSync(this.native, filepath, name);
        }

        reset() {
            return nativeModule.LuaState_reset(this.native);
        }

        close() {
            return nativeModule.LuaState_close(this.native);
        }

        getGlobal(name) {
            return nativeModule.LuaState_getGlobal(this.native, name);
        }

        setGlobal(name, value) {
            return nativeModule.LuaState_setGlobal(this.native, name, value);
        }

        registerEventListener(name, f) {
            return nativeModule.LuaState_registerEventListener(this.native, name, f);
        }
    }(stateOptions)
}
exports.createLuaState = createLuaState;

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