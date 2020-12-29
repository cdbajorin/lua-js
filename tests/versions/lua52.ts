import anyTest, { afterEach, beforeEach, TestInterface } from "ava";

import { toLuaSuite } from "../suites/to-lua";
import { createLuaState, Lua52 } from "../../lib";
import { Context } from "../index";
import { fromLuaSuite } from "../suites/from-lua";
import { apiSuite } from "../suites/api";

const test = anyTest as TestInterface<Context>;

beforeEach(t => {
    t.context = {
        lua: createLuaState({ version: Lua52.Version })
    }
});

afterEach(t => {
    (t.context as Context).lua.close();
});

/**
 * Test suites
 */
test("lua52", toLuaSuite);
test("lua52", fromLuaSuite);
test("lua52", apiSuite);