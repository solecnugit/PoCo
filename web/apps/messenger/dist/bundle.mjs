import express from 'express';
import http from 'node:http';
import chalk from 'chalk';

const app = express();
const server = http.createServer(app);
const port = 8080;
app.get("/", (req, res) => {
    res.send("Hello");
    res.send(process.env.version);
});
server.listen(port, () => {
    console.log("server is running at port", chalk.green(port.toString()));
});
//# sourceMappingURL=bundle.mjs.map
