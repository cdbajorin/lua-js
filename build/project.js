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
exports.Project = void 0;
const path = __importStar(require("path"));
const feature_1 = require("./feature");
const cp = __importStar(require("child_process"));
const ROOT = path.resolve(__dirname, "..");
const buildTarget = async (crate, release) => {
    return new Promise((resolve, reject) => {
        const cwd = crate.root;
        const flags = ["build"].concat((release ? ["--release"] : []));
        let ps = cp.spawn("cargo", flags, { cwd, stdio: "inherit" });
        ps.on("error", reject);
        ps.on("close", (num) => {
            const target = release ? "release" : "debug";
            // TODO this is OSX-only, refactor for windows/linux support.
            const dylibName = `lib${crate.name}.dylib`;
            const dylib = path.resolve(ROOT, "native", "target", target, dylibName);
            resolve(dylib);
        });
    });
};
class Project {
    constructor(featureCrates) {
        this.featureCrates = featureCrates;
    }
    static create(features) {
        let featureCrates = features.reduce((acc, feature) => {
            acc.push(new feature_1.FeatureCrate(feature));
            return acc;
        }, []);
        return new Project(featureCrates);
    }
    async build(release) {
        for (let crate of this.featureCrates) {
            let dylib = await buildTarget(crate, release);
            await crate.finish(dylib);
        }
    }
}
exports.Project = Project;
