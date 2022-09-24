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
            sourcemap: true,
            name: 'pocoNet',
            globals: {
                "socket.io-client": "socket_ioClient",
                "lodash": "_"
            }
        },
        {
            file: "dist/bundle.mjs",
            format: "esm",
            sourcemap: true,
            globals: {
                "socket.io-client": "socket_ioClient",
                "lodash": "_"
            }
        },
    ],
    plugins: [
        json(),
        externals({
            devDeps: false,
        }),
        replace({
            preventAssignment: true,
            __PROTOCOL_VERSION__: JSON.stringify("poco-0.1")
        }),
        resolve(),
        commonjs(),
        typescript(),
        strip(),
    ],
    watch: {
        exclude: "node_modules/**",
    },
});