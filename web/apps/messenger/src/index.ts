import express from "express";
import http from "http";
import chalk from "chalk";
import { Server as SocketIOServer } from "socket.io";

const app = express();
const server = http.createServer(app);
const io = new SocketIOServer(server);
const port = 8080;

app.get("/", (req, res) => {
    res.send("Hello")
    res.send(process.env.version)
})

io.on("connection", (socket) => {
    console.log("A user connected.")
    socket.on("disconnect", () => {
        console.log("A user disconnected.")
    })
})

server.listen(port, () => {
    console.log("server is running at port", chalk.green(port.toString()))
})