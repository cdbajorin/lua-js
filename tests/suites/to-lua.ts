import { Macro, OneOrMoreMacros } from "ava";
import { Context } from "../index";

// TODO BigInt/Symbol
export const toLuaNumber: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    state.setGlobal("one", 1);
    const luaAssertOne = () => {
        state.doStringSync("assert(one == 1)")
    };
    t.notThrows(luaAssertOne);

    state.setGlobal("negativeOne", -1);
    const luaAssertNegativeOne = () => {
        state.doStringSync("assert(negativeOne == -1)")
    };
    t.notThrows(luaAssertNegativeOne);

    state.setGlobal("maxInt", Number.MAX_SAFE_INTEGER);
    const luaAssertMaxInt = () => {
        state.doStringSync(`assert(maxInt == ${Number.MAX_SAFE_INTEGER})`)
    };
    t.notThrows(luaAssertMaxInt);

    state.setGlobal("maxNegInt", -Number.MAX_SAFE_INTEGER);
    const luaAssertMaxNegInt = () => {
        state.doStringSync(`assert(maxNegInt == -${Number.MAX_SAFE_INTEGER})`)
    };
    t.notThrows(luaAssertMaxNegInt);
}
toLuaNumber.title = (version) => `${version}: ToLua: It converts numbers`;

export const toLuaString: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const greeting = "Hello World!";
    state.setGlobal("greeting", greeting);
    const luaAssertString = () => {
        state.doStringSync(`assert(greeting == '${greeting}')`)
    };
    t.notThrows(luaAssertString);
}
toLuaString.title = (version) => `${version}: ToLua: It converts strings`;

export const toLuaBool: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const boolTrue = true;
    state.setGlobal("boolTrue", boolTrue);
    const luaAssertTrue = () => {
        state.doStringSync(`assert(boolTrue == true)`);
    };
    t.notThrows(luaAssertTrue);

    const boolFalse = false;
    state.setGlobal("boolFalse", boolFalse);
    const luaAssertFalse = () => {
        state.doStringSync(`assert(boolFalse == false)`);
    };
    t.notThrows(luaAssertFalse);
}
toLuaBool.title = (version) => `${version}: ToLua: It converts booleans`;


export const toLuaNullUndefined: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const jsUndefined = undefined;
    state.setGlobal("jsUndefined", jsUndefined);
    const luaAssertUndefined = () => {
        state.doStringSync(`assert(boolTrue == nil)`);
    };
    t.notThrows(luaAssertUndefined);

    const jsNull = null;
    state.setGlobal("jsNull", jsNull);
    const luaAssertNull = () => {
        state.doStringSync(`assert(jsNull == nil)`);
    };
    t.notThrows(luaAssertNull);
}
toLuaNullUndefined.title = (version) => `${version}: ToLua: It converts undefined/null`;


export const toLuaArray: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const jsArray = [1, 2, 3];
    state.setGlobal("jsArray", jsArray);
    const luaAssertArray = () => {
        state.doStringSync(`
        assert(jsArray[1] == 1)
        assert(jsArray[2] == 2)
        assert(jsArray[3] == 3)
        assert(jsArray[4] == nil)
        `);
    };
    t.notThrows(luaAssertArray);
}
toLuaArray.title = (version) => `${version}: ToLua: It converts arrays`;

// TODO unimplemented
export const toLuaObject: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const jsObject = {
        a: 1,
        b: 2,
        c: 3
    };
    state.setGlobal("jsObject", jsObject);
    const luaAssertArray = () => {
        state.doStringSync(`
        assert(jsObject.a == 1)
        assert(jsObject.b == 2)
        assert(jsObject.c == 3)
        `);
    };
    t.notThrows(luaAssertArray);
}
toLuaObject.title = (version) => `${version}: ToLua: It converts Objects`;

export const toLuaSparseArray: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const tableLike: any = [1, null, 3];
    state.setGlobal("tableLike", tableLike);
    const luaAssertArray = () => {
        state.doStringSync(`
        assert(tableLike[1] == 1)
        assert(tableLike[2] == nil)
        assert(tableLike[3] == 3)
        `);
    };
    t.notThrows(luaAssertArray);
    const tableLike2: any = [1, undefined, 3];
    state.setGlobal("tableLike2", tableLike2);
    const luaAssertArray2 = () => {
        state.doStringSync(`
        assert(tableLike2[1] == 1)
        assert(tableLike2[2] == nil)
        assert(tableLike2[3] == 3)
        `);
    };
    t.notThrows(luaAssertArray2);
}
toLuaSparseArray.title = (version) => `${version}: ToLua: It converts sparse arrays`;

export const toLuaSuite: OneOrMoreMacros<[], Context> = [
    toLuaNumber,
    toLuaString,
    toLuaBool,
    toLuaNullUndefined,
    toLuaArray,
    toLuaObject,
    toLuaSparseArray
]
