import { defineConfig } from "rollup";
import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs"
import json from "@rollup/plugin-json"
import externals from "rollup-plugin-node-externals";
import strip from "@rollup/plugin-strip";
import replace from "@rollup/plugin-replace";
import babel from "@rollup/plugin-babel";
import * as pkg from "./package.json";

const globals = {
    "@truffle/contract": "contract",
    "web3": "web3"
}

export default defineConfig({
    input: "./src/index.ts",
    output: [
        {
            file: "./dist/index.umd.js",
            format: "umd",
            sourcemap: true,
            name: pkg.name,
            globals
        },
        {
            file: "./dist/index.esm.js",
            format: "esm",
            sourcemap: true,
        },
        {
            file: "./dist/index.cjs.js",
            format: "cjs",
            sourcemap: true
        }
    ],
    external: [
        "@truffle/contract",
        "web3"
    ],
    plugins: [
        json(),
        resolve({
            browser: true
        }),
        commonjs({
            sourceMap: false
        }),
        typescript({
            sourceMap: true,
        }),
        externals({
            devDeps: false,
        }),
        replace({
            preventAssignment: true,
            __POCO_PROTOCOL_VERSION__: JSON.stringify("poco-alpha")
        }),
        babel({ babelHelpers: "bundled" }),
    ],
    watch: {
        exclude: "node_modules/**",
    },
});