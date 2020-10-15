export class LuaState {

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
    registerFunction<T extends []>(name: string, cb: (...args: T) => void): void


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