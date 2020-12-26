import { createLuaState, Lua54, Lua53, Lua52, Lua51, LuaJIT, LuaState } from "../lib";
import test from "ava";

export type Context = {
    lua: LuaState
}

/**
 * Load correct versions
 */
test("It loads Lua 5.4", (t) => {
    const state = createLuaState({ version: Lua54.Version })
    let actual = state.getGlobal("_VERSION");
    t.is(actual, "Lua 5.4");
})

test("It loads Lua 5.3", (t) => {
    const state = createLuaState({ version: Lua53.Version })
    let actual = state.getGlobal("_VERSION");
    t.is(actual, "Lua 5.3");
})

test("It loads Lua 5.2", (t) => {
    const state = createLuaState({ version: Lua52.Version })
    let actual = state.getGlobal("_VERSION");
    t.is(actual, "Lua 5.2");
})

test("It loads Lua 5.1", (t) => {
    const state = createLuaState({ version: Lua51.Version })
    let actual = state.getGlobal("_VERSION");
    t.is(actual, "Lua 5.1");
})

test("It loads Lua JIT", (t) => {
    const state = createLuaState({ version: LuaJIT.Version })
    let actual = state.getGlobal<{ version: string }>("jit");
    t.is(actual.version, "LuaJIT 2.1.0-beta3");
})

/**
 * Library loading
 */
test("It loads libraries", (t) => {
    const state = createLuaState({ version: Lua54.Version, libraries: [Lua54.Libs.math] });
    const runNoError = () => {
        state.doStringSync("math.floor(1.5)");
    }
    t.notThrows(runNoError)
})

test("It errors when using non-loaded libraries", (t) => {
    const state = createLuaState({ version: Lua54.Version, libraries: [] })
    const runError = () => {
        state.doStringSync("math.floor(1.5)");
    }
    t.throws(runError);
})