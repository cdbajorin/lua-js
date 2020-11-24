import * as path from "path";
import * as fs from "fs";
import { promisify } from "util";
import _rimraf from "rimraf";

const rimraf = promisify(_rimraf);
const copyFile = promisify(fs.copyFile);

const ROOT = path.resolve(__dirname, "..");
const NATIVE = path.resolve(ROOT, "native");

export class FeatureCrate {
    name: string;
    root: string;
    addon: string;

    constructor(name: string) {
        this.name = name;
        this.root = path.resolve(NATIVE, name);
        this.addon = path.resolve(NATIVE, `${name}.node`);
    }

    async finish(dylib: string): Promise<void> {
        await rimraf(this.addon);
        await copyFile(dylib, this.addon)
    }
}