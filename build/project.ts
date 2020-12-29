// import * as path from "path";
import { FeatureCrate } from "./feature";
import * as cp from "child_process";

// const ROOT = path.resolve(__dirname, "..");

const buildTarget = async (crate: FeatureCrate, release: boolean): Promise<number> => {
    return new Promise((resolve,reject) => {
        const cwd = crate.root;
        const flags = ["build"].concat((release ? ["--release"] : []));
        let ps = cp.spawn("cargo", flags, { cwd, stdio: "inherit" });
        ps.on("error", reject);
        ps.on("close", resolve);
        // ps.on("close", (num) => {
        //     const target = release ?  "release" : "debug";
        //     // TODO this is OSX-only, refactor for windows/linux support.
        //     const dylibName = `lib${crate.name}.dylib`;
        //     const dylib = path.resolve(ROOT, "native", "target", target, dylibName);
        //     resolve(dylib)
        // });
    });
};

export class Project {
    featureCrates: FeatureCrate[];

    constructor(featureCrates: FeatureCrate[]) {
        this.featureCrates = featureCrates;
    }

    static create(features: string[]) {
        let featureCrates = features.reduce((acc: FeatureCrate[], feature) => {
            acc.push(new FeatureCrate(feature))
            return acc;
        }, []);
        return new Project(featureCrates);
    }

    async build(release: boolean) {

        for (let crate of this.featureCrates) {
            await buildTarget(crate, release);
            // await crate.finish(dylib);
        }
    }
}