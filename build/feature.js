"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.FeatureCrate = void 0;
const path = __importStar(require("path"));
// import * as fs from "fs";
// import { promisify } from "util";
// import _rimraf from "rimraf";
// const rimraf = promisify(_rimraf);
// const copyFile = promisify(fs.copyFile);
const ROOT = path.resolve(__dirname, "..");
const NATIVE = path.resolve(ROOT, "native");
class FeatureCrate {
    // addon: string;
    constructor(name) {
        this.name = name;
        this.root = path.resolve(NATIVE, name);
        // this.addon = path.resolve(NATIVE, `${name}.node`);
    }
}
exports.FeatureCrate = FeatureCrate;
