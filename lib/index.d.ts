interface SharedLibs {
    table: 0x2,
    io: 0x4,
    os: 0x8,
    string: 0x10,
    math: 0x80,
    package: 0x100,
    debug: 0x80000000,
    ALL: 0xFFFFFFFF,
    // Excludes debug and ffi
    ALL_SAFE: 0xFFFFFFFE,
}

interface CoroutineLib {
    coroutine: 0x1;
}

interface Utf8Lib {
    utf8: 0x20;
}

interface BitLib {
    bit: 0x40;
}

interface JitLib {
    jit: 0x200;
    ffi: 0x40000000,
}

interface Lua51Libs extends SharedLibs {
}

interface Lua52Libs extends SharedLibs, CoroutineLib, BitLib {
}

interface Lua53Libs extends SharedLibs, CoroutineLib, Utf8Lib {
}

interface Lua54Libs extends Lua53Libs {
}

interface LuaJitLibs extends SharedLibs, BitLib, JitLib {
}

interface Lua51 {
    Version: "lua51",
    Libs: Lua51Libs
}

interface Lua52 {
    Version: "lua52",
    Libs: Lua52Libs
}

interface Lua53 {
    Version: "lua53",
    Libs: Lua53Libs
}

interface Lua54 {
    Version: "lua54",
    Libs: Lua54Libs
}

interface LuaJit {
    Version: "luajit",
    Libs: LuaJitLibs
}

type Values<K> = (K[keyof K])[]

type LuaStateOptions =
    | { version: "lua51"; libraries?: Values<Lua51Libs>; }
    | { version: "lua52"; libraries?: Values<Lua52Libs>; }
    | { version: "lua53"; libraries?: Values<Lua53Libs>; }
    | { version: "lua54"; libraries?: Values<Lua54Libs>; }
    | { version: "luajit"; libraries?: Values<LuaJitLibs>; }

/**
 * If `libraries` field is excluded, it defaults to `ALL_SAFE`
 */
export function createLuaState(options: LuaStateOptions): LuaState;

export const Lua51: Lua51;
export const Lua52: Lua52;
export const Lua53: Lua53;
export const Lua54: Lua54;
export const LuaJIT: LuaJit;

/**
 * Collection of symbols used by lua-js
 */
export const Symbols: {
    /**
     * Representation for using `0` as a property key in Lua:
     * `{ [0] = "foo" }` will convert to `{ [Symbols.ZeroIndex]: "foo" }`
     */
    ZeroIndex: symbol
}

export class LuaState {

    /**
     * Executes a string of code synchronously.
     *
     * @param code
     * @param chunkName
     */
    doStringSync<T extends any[]>(code: string, chunkName?: string): T;

    /**
     * Set a lua global function which calls back to the JS runtime as an event emitter
     *
     * @param name
     * @param cb
     */
    registerEventListener<T extends any[]>(name: string, cb: (...args: T) => void): void;

    /**
     * Executes a Lua file synchronously.
     *
     * @param name
     * @param chunkName
     */
    doFileSync<T extends any[]>(name: string, chunkName?: string): T;

    // /**
    //  * Calls a function-like chunk of code:
    //  *
    //  * ```
    //  * lua.callChunk('function(a,b) return a + b end', 1, 2) === 3
    //  * lua.callChunk('tostring', 1) === "1.0"
    //  * ```
    //  *
    //  * @param code
    //  * @param args
    //  */
    // callChunk<T extends any[], R>(code: string, args: T): R;
    // callChunk<T extends any[], R>(code: string, chunkName: string, args: T): R;

    // /**
    //  * @async
    //  * @name doString
    //  * @param code {string}
    //  * @param cb{Function(T) => void}
    //  */
    // doString<T extends []>(code: string, cb: (resultSet: T) => void): void;
    //
    // /**
    //  * @param name {string}
    //  * @param cb {Function(T) => void}
    //  */
    // addHook<T extends []>(name: string): void;
    //
    // /**
    //  *
    //  * @param name
    //  * @param string
    //  * @param args
    //  */
    // callFunction<T extends []>(name, string, ...args: T): void;

    /**
     * Sets a global in the *current* context. calling reset() will
     * wipe the global from the state.
     *
     * @name setGlobal
     * @param name {string}
     * @param value
     */
    setGlobal(name: string, value: any): void;

    /**
     * Get a global from the current context.
     *
     * @name getGlobal
     * @param name {string}
     * @returns {T}
     */
    getGlobal<T>(name: string): T;

    /**
     * This is a mutable reset. It closes the internal Lua context, spawning a
     * new one. This clears all globals, as well as clears the event emitters
     * hidden behind registerFunction. Since it clears the event emitters it is
     * also effectively `close()`, and allows the current program to exit.
     */
    reset(): void;

    /**
     * Alias for `reset()`
     */
    close(): void;
}
