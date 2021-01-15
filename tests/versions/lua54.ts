import anyTest, { afterEach, beforeEach, TestInterface } from "ava";

import { toLuaSuite } from "../suites/to-lua";
import { createLuaState, Lua54 } from "../../lib";
import { Context } from "../index";
import { fromLuaSuite } from "../suites/from-lua";
import { apiSuite, cbApiSuite } from "../suites/api";

const test = anyTest as TestInterface<Context>;

beforeEach(t => {
    t.context = {
        lua: createLuaState({ version: Lua54.Version })
    }
});

afterEach(t => {
    (t.context as Context).lua.close();
});

/**
 * Test suites
 */
test("lua54", toLuaSuite);
test("lua54", fromLuaSuite);
test("lua54", apiSuite);
test.cb("lua54", cbApiSuite);