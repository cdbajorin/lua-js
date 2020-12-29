import anyTest, { afterEach, beforeEach, TestInterface } from "ava";

import { toLuaSuite } from "../suites/to-lua";
import { createLuaState, LuaJIT } from "../../lib";
import { Context } from "../index";
import { fromLuaSuite } from "../suites/from-lua";
import { apiSuite } from "../suites/api";

const test = anyTest as TestInterface<Context>;

beforeEach(t => {
    t.context = {
        lua: createLuaState({ version: LuaJIT.Version })
    }
});

afterEach(t => {
    (t.context as Context).lua.close();
});

/**
 * Test suites
 */
test("luajit", toLuaSuite);
test("luajit", fromLuaSuite);
test("luajit", apiSuite);