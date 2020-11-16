export const enum Version {
    Lua54 = "lua54",
    Lua53 = "lua53",
    Lua52 = "lua52",
    Lua51 = "lua51",
    LuaJIT = "luajit",
}

export const enum LuaLib {
    // Lua54/Lua53/Lua52
    Coroutine = 0x1,
    Table = 0x2,
    Io = 0x4,
    Os = 0x8,
    String = 0x10,
    // Lua54/Lua53
    Utf8 = 0x20,
    // Lua52/LuaJIT
    Bit = 0x40,
    Math = 0x80,
    Package = 0x100,
    // LuaJIT
    Jit = 0x200,
    // LuaJIT
    Ffi = 0x40000000,
    Debug = 0x80000000,
    All = 0xFFFFFFFF,
    // Excludes Ffi/Debug
    AllSafe = 0xFFFFFFFE,
}

export interface LuaStateOptions {
    libraries: LuaLib[];
}

export class LuaState {

    constructor(options?: LuaStateOptions);

    /**
     * Executes a string of code synchronously.
     *
     * @param code
     * @param chunkName
     */
    doStringSync(code: string, chunkName?: string): undefined;

    // doStringSync<T extends []>(code: string, chunkName?: string): T;

    /**
     * This is more like an event emitter than a callback. It can be invoked
     * multiple times. Event Emitters were more difficult to construct than
     * a reusable callback. This may change in the future.
     *
     * @param name
     * @param cb
     */
    registerFunction<T extends any[]>(name: string, cb: (args: T) => void): void


    /**
     * Executes a Lua file synchronously.
     *
     * @param name
     * @param chunkName
     */
    doFileSync(name: string, chunkName?: string): undefined;

    /**
     * Calls a function-like chunk of code:
     *
     * ```
     * state.callChunk('function(a,b) return a + b end', 1, 2) === 3
     * state.callChunk('tostring', 1) === "1.0"
     * ```
     *
     * @param code
     * @param args
     */
    callChunk<T extends any[], R>(code: string, args: T): R;
    callChunk<T extends any[], R>(code: string, chunkName: string, args: T): R;
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
    getGlobal<T>(name: string): T | undefined;

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
