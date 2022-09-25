import { defineConfig } from "rollup";
import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs"
import json from "@rollup/plugin-json"
import externals from "rollup-plugin-node-externals";
import strip from "@rollup/plugin-strip";
import replace from "@rollup/plugin-replace"

export default defineConfig({
    input: "./src/index.ts",
    output: [
        {
            file: "dist/bundle.cjs",
            format: "umd",
            sourcemap: "inline",
            name: 'pocoNet',
            globals: {
                "socket.io-client": "socket_ioClient",
                "lodash": "_"
            },
            compact: false
        },
        {
            file: "dist/bundle.mjs",
            format: "esm",
            sourcemap: "inline",
            globals: {
                "socket.io-client": "socket_ioClient",
                "lodash": "_"
            },
            compact: false
        },
    ],
    plugins: [
        replace({
            preventAssignment: true,
            __PROTOCOL_VERSION__: JSON.stringify("poco-0.1")
        }),
        json(),
        resolve(),
        typescript({
            sourceMap: true,
        }),
        commonjs({
            sourceMap: true
        }),
        externals({
            devDeps: false,
        }),
        process.env.development ? strip() : undefined,
    ],
    watch: {
        exclude: "node_modules/**",
    },
});