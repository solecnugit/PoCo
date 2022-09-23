(function (global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? factory(require('express'), require('node:http'), require('chalk')) :
    typeof define === 'function' && define.amd ? define(['express', 'node:http', 'chalk'], factory) :
    (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global.express, global.http, global.chalk));
})(this, (function (express, http, chalk) { 'use strict';

    function _interopDefaultLegacy (e) { return e && typeof e === 'object' && 'default' in e ? e : { 'default': e }; }

    var express__default = /*#__PURE__*/_interopDefaultLegacy(express);
    var http__default = /*#__PURE__*/_interopDefaultLegacy(http);
    var chalk__default = /*#__PURE__*/_interopDefaultLegacy(chalk);

    const app = express__default["default"]();
    const server = http__default["default"].createServer(app);
    const port = 8080;
    app.get("/", (req, res) => {
        res.send("Hello");
        res.send(process.env.version);
    });
    server.listen(port, () => {
        console.log("server is running at port", chalk__default["default"].green(port.toString()));
    });

}));
//# sourceMappingURL=bundle.cjs.map
