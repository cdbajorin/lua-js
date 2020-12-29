import { Context } from "../index";
import { Macro, OneOrMoreMacros } from "ava";
import * as path from "path";

export const doFileSync: Macro<[], Context> = (t) => {

    const fivePlusFive = path.resolve(__dirname, "files", "five_plus_five.lua");
    const state = t.context.lua;
    const actual = state.doFileSync(fivePlusFive);
    t.deepEqual(actual, [10]);
}
doFileSync.title = (version) => `${version}: API: doFileSync`;

export const doStringSync: Macro<[], Context> = (t) => {
    const lua = t.context.lua;
    const actual = lua.doStringSync("return 5 + 5");
    t.deepEqual(actual, [10]);
}
doStringSync.title = (version) => `${version}: API: doStringSync`;

export const globals: Macro<[], Context> = (t) => {
    const lua = t.context.lua;
    lua.setGlobal("value", 10);
    const actual = lua.getGlobal("value");
    t.is(actual, 10);
}
globals.title = (version) => `${version}: API: set/get globals`;

export const apiSuite: OneOrMoreMacros<[], Context> = [
    doFileSync,
    doStringSync,
    globals
]