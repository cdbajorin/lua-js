import { Macro, OneOrMoreMacros } from "ava";
import { Context } from "../index";
import { Symbols } from "../../lib";

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
fromLuaNumber.title = (version) => `${version}: FromLua: It converts numbers`;

export const fromLuaString: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const expected = "Hello World!";
    state.doStringSync(`greeting = "${expected}"`);
    const actual = state.getGlobal("greeting");
    t.is(actual, expected);
}
fromLuaString.title = (version) => `${version}: FromLua: It converts strings`;

export const fromLuaBool: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync("boolTrue = true");
    const boolTrue = state.getGlobal("boolTrue");
    t.is(boolTrue, true);

    state.doStringSync("boolFalse = false");
    const boolFalse = state.getGlobal("boolFalse");
    t.is(boolFalse, false);
}
fromLuaBool.title = (version) => `${version}: FromLua: It converts bools`;

export const fromLuaNil: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`value = nil`);
    const actual = state.getGlobal("value");
    t.is(actual, undefined);
}
fromLuaNil.title = (version) => `${version}: FromLua: It converts nil`;

export const fromLuaArrayTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`myArray = { 1, 2, 3 }`);
    const actual = state.getGlobal("myArray");
    t.deepEqual(actual, [1, 2, 3]);
}
fromLuaArrayTable.title = (version) => `${version}: FromLua: It converts array-tables`;

export const fromLuaSparseArrayTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`myArray = { 1, 2, nil, 3 }`);
    const actual = state.getGlobal("myArray");
    t.deepEqual(actual, [1, 2, undefined, 3]);
}
fromLuaSparseArrayTable.title = (version) => `${version}: FromLua: It converts sparse array-tables`;

export const fromLuaObjectTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    // language=Lua
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
fromLuaObjectTable.title = (version) => `${version}: FromLua: It converts object-tables`;

export const fromLuaNegativeIndexableKeys: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    // language=Lua
    state.doStringSync(`
        myObj = {
            [-1] = -1,
            ["-1"] = "-1"
        }
    `);
    const actual = state.getGlobal("myObj");
    t.deepEqual(actual, { [-1]: -1, ["'-1'"]: "-1" });
}
fromLuaNegativeIndexableKeys.title = (version) => `${version}: FromLua: It converts out of range integers keys into an object`;

export const fromLuaIndexableKeys: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    // language=Lua
    state.doStringSync(`
        myObj = {
            [1] = 1,
            ["1"] = "1"
        }
    `);
    const actual = state.getGlobal("myObj");
    const expected: any = [1];
    expected["'1'"] = "1";
    t.deepEqual(actual, expected);
}
fromLuaIndexableKeys.title = (version) => `${version}: FromLua: it wraps integer-representable strings keys in quotes`;


export const fromLuaZeroIndexTable: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    // language=Lua
    state.doStringSync(`
        myObj = {
            [0] = "LuaJs.0",
            ["0"] = "0",
            0
        }
    `);
    const actual = state.getGlobal("myObj");
    const expected: any = [0];
    expected["'0'"] = "0";
    expected[Symbols.ZeroIndex] = "LuaJs.0"
    t.deepEqual(actual, expected);
}
fromLuaZeroIndexTable.title = (version) => `${version}: FromLua: it handles zero-index property keys`;

export const fromLuaObjectPropKeyError: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    // language=Lua
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
fromLuaObjectPropKeyError.title = (version) => `${version}: FromLua: It errors converting invalid JS property keys`;

export const fromLuaPropKeyGTu32Max: Macro<[], Context> = (t) => {
    const U32_MAX = 4294967295;
    const state = t.context.lua;
    // 1-based index conversion means we need to add 2 to force it out of range.
    // language=Lua
    state.doStringSync(`
        arrLike = {
            [${U32_MAX + 1}] = "MAX"
        }
        objLike = {
            [${U32_MAX + 2}] = "u32_max plus 2"
        }
    `);
    const actualArr = state.getGlobal("arrLike");
    const expectedArr = [];
    expectedArr[U32_MAX] = "MAX";
    t.deepEqual(actualArr, expectedArr);
    const actualObj = state.getGlobal("objLike");
    t.deepEqual(actualObj, { [`${U32_MAX + 2}`]: "u32_max plus 2" });
}
fromLuaPropKeyGTu32Max.title = (version) => `${version}: FromLua: It converts array indices out of JS range to string keys`;

export const fromLuaFunction: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.doStringSync(`function noop() end`);
    const actual = state.getGlobal("noop");
    t.is(actual, "[LuaFunction]")
}
fromLuaFunction.title = (version) => `${version}: FromLua: It stringifies functions`;

export const fromLuaSuite: OneOrMoreMacros<[], Context> = [
    fromLuaNumber,
    fromLuaString,
    fromLuaBool,
    fromLuaNil,
    fromLuaArrayTable,
    fromLuaSparseArrayTable,
    fromLuaObjectTable,
    fromLuaZeroIndexTable,
    fromLuaObjectPropKeyError,
    fromLuaNegativeIndexableKeys,
    fromLuaIndexableKeys,
    fromLuaPropKeyGTu32Max,
    fromLuaFunction,
];
