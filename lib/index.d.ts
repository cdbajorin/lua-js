export const enum LuaLib {
    Base = 1,
    Coroutine = 1 << 1,
    Table = 1 << 2,
    Io = 1 << 3,
    Os = 1 << 4,
    String = 1 << 5,
    Utf8 = 1 << 6,
    Math = 1 << 7,
    Package = 1 << 8,
    Debug = 1 << 9,
}

/**
 * Lua library sets to be included when instantiating a new state.
 * Default: `AllNoDebug`
 * The only lib set that includes `debug` is `All`.
 */
interface LuaLibSet {
    All: LuaLib[];
    AllNoDebug: LuaLib[];
    /**
     * Excludes io/os/debug
     */
    NoFs: LuaLib[];
}

export const LuaLibSet: LuaLibSet;

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

    // doFileSync<T extends []>(name: string, chunkName?: string): T;

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