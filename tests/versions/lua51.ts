import anyTest, { afterEach, beforeEach, TestInterface } from "ava";

import { toLuaSuite } from "../suites/to-lua";
import { createLuaState, Lua51 } from "../../lib";
import { Context } from "../index";
import { fromLuaSuite } from "../suites/from-lua";
import { apiSuite } from "../suites/api";

const test = anyTest as TestInterface<Context>;

beforeEach(t => {
    t.context = {
        lua: createLuaState({ version: Lua51.Version })
    }
});

afterEach(t => {
    (t.context as Context).lua.close();
});

/**
 * Test suites
 */
test("lua51", toLuaSuite);
test("lua51", fromLuaSuite);
test.only("lua51", apiSuite);
