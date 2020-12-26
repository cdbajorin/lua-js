import { Macro, OneOrMoreMacros } from "ava";
import { Context } from "../index";

// TODO BigInt
export const fromLuaNumber: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync("one = 1");
    const one = state.getGlobal("one");
    t.is(one, 1);

    state.doStringSync("negOne = -1");
    const negOne = state.getGlobal("negOne");
    t.is(negOne, -1);

    state.doStringSync(`maxSafeInt = ${Number.MAX_SAFE_INTEGER}`);
    const maxSafe = state.getGlobal("maxSafeInt");
    t.is(maxSafe, Number.MAX_SAFE_INTEGER);

    state.doStringSync(`minSafeInt = ${Number.MIN_SAFE_INTEGER}`);
    const minSafe = state.getGlobal("minSafeInt");
    t.is(minSafe, Number.MIN_SAFE_INTEGER);
}
fromLuaNumber.title = (version) => `${version}: It converts numbers from Lua`;

export const fromLuaString: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const expected = "Hello World!";
    state.doStringSync(`greeting = "${expected}"`);
    const actual = state.getGlobal("greeting");
    t.is(actual, expected);
}
fromLuaString.title = (version) => `${version}: It converts strings from Lua`;

export const fromLuaBool: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync("boolTrue = true");
    const boolTrue = state.getGlobal("boolTrue");
    t.is(boolTrue, true);

    state.doStringSync("boolFalse = false");
    const boolFalse = state.getGlobal("boolFalse");
    t.is(boolFalse, false);
}
fromLuaBool.title = (version) => `${version}: It converts bools from Lua`;

export const fromLuaNil: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`value = nil`);
    const actual = state.getGlobal("value");
    t.is(actual, undefined);
}
fromLuaNil.title = (version) => `${version}: It converts nil from Lua`;

export const fromLuaArrayTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`myArray = { 1, 2, 3 }`);
    const actual = state.getGlobal("myArray");
    t.deepEqual(actual, [1, 2, 3]);
}
fromLuaArrayTable.title = (version) => `${version}: It converts array-tables from Lua`;

export const fromLuaSparseArrayTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`myArray = { 1, 2, nil, 3 }`);
    const actual = state.getGlobal("myArray");
    t.deepEqual(actual, [1, 2, undefined, 3]);
}
fromLuaSparseArrayTable.title = (version) => `${version}: It converts sparse array-tables from Lua`;

export const fromLuaObjectTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`
    a = "a"
        myObj = {
         [a] = 1, 
         b = 2, 
         c = 3, 
         [1.5] = 4
       }
   `);
    const actual = state.getGlobal("myObj");
    t.deepEqual(actual, { a: 1, b: 2, c: 3, ["1.5"]: 4 });
}
fromLuaObjectTable.title = (version) => `${version}: It converts object-tables from Lua`;

export const fromLuaObjectPropKeyError: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`
        obj = {
         [{}] = 1
       }
   `);
    const shouldError = () => {
        const _ = state.getGlobal("obj");
    }
    t.throws(shouldError)
}
fromLuaObjectPropKeyError.title = (version) => `${version}: It errors converting invalid JS property keys from Lua`;

export const fromLuaFunction: Macro<[],Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`function noop() end`);
    const actual = state.getGlobal("noop");
    t.is(actual, "[LuaFunction]")
}
fromLuaFunction.title = (version) => `${version}: It stringifies functions from Lua`;

export const fromLuaSuite: OneOrMoreMacros<[], Context> = [
    fromLuaNumber,
    fromLuaString,
    fromLuaBool,
    fromLuaNil,
    fromLuaArrayTable,
    fromLuaSparseArrayTable,
    fromLuaObjectTable,
    fromLuaObjectPropKeyError,
    fromLuaFunction
];