import { Macro, OneOrMoreMacros } from "ava";
import { Context } from "../index";

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
toLuaNumber.title = (version) => `${version}: It converts numbers to Lua`;

export const toLuaString: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const greeting = "Hello World!";
    state.setGlobal("greeting", greeting);
    const luaAssertString = () => {
        state.doStringSync(`assert(greeting == '${greeting}')`)
    };
    t.notThrows(luaAssertString);
}
toLuaString.title = (version) => `${version}: It converts strings to Lua`;

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
toLuaBool.title = (version) => `${version}: It converts bools to Lua`;


export const toLuaNil: Macro<[], Context> = (t) => {
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
toLuaNil.title = (version) => `${version}: It converts undefined/null to Lua`;


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
toLuaArray.title = (version) => `${version}: It converts arrays to Lua`;

// TODO unimplemented
export const toLuaTable: Macro<[], Context> = (t) => {
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
toLuaTable.title = (version) => `${version}: It converts Objects to Lua`;

export const toLuaTableMixed: Macro<[], Context> = (t) => {
    const state = t.context.lua;
    const tableLike: any = [1, 2, 3];
    tableLike.propA = "abc";
    tableLike[1.5] = "things";
    state.setGlobal("tableLike", tableLike);
    const luaAssertArray = () => {
        state.doStringSync(`
        assert(tableLike[1] == 1)
        assert(tableLike[2] == 2)
        assert(tableLike[3] == 3)
        assert(tableLike.propA == "abc")
        `);
    };
    t.notThrows(luaAssertArray);
}
toLuaTableMixed.title = (version) => `${version}: It converts arrays with properties to Lua`;

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
}
toLuaSparseArray.title = (version) => `${version}: It converts sparse arrays to Lua`;

export const toLuaSuite: OneOrMoreMacros<[], Context> = [
    toLuaNumber,
    toLuaString,
    toLuaBool,
    toLuaNil,
    toLuaArray,
    // TODO Lua Tables. Needs some work on mapping keys to indexes/fields
    toLuaTable,
    toLuaTableMixed,
    toLuaSparseArray
]
