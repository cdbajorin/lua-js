import anyTest, { afterEach, beforeEach, TestInterface } from "ava";

import { toLuaSuite } from "../suites/to-lua";
import { createLuaState, Lua53 } from "../../lib";
import { Context } from "../index";
import { fromLuaSuite } from "../suites/from-lua";
import { apiSuite } from "../suites/api";

const test = anyTest as TestInterface<Context>;

beforeEach(t => {
    t.context = {
        lua: createLuaState({ version: Lua53.Version })
    }
});

afterEach(t => {
    (t.context as Context).lua.close();
});

/**
 * Test suites
 */
test("lua53", toLuaSuite);
test("lua53", fromLuaSuite);
test("lua53", apiSuite);