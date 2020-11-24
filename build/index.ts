import { Project } from "./project";

const project = Project.create(["lua54", "lua53", "lua52", "lua51", "luajit"])

const flags = process.argv.slice(2);
const isRelease = flags[0] == "--release"

project.build(isRelease)
    .then(() => process.exit(0))
    .catch((e) => {
        console.error(e);
        process.exit(1)
    })