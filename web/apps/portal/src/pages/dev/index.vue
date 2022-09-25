<script setup lang="ts">
import { $ref } from "vue/macros"
import * as poco from "poco-net"

let localAddress = $ref("")
let remoteAddress = $ref("")
let history = $ref("");
let message = $ref("");

let socket: poco.net.PocoConnection;
let peer: poco.net.PocoPeerConnection;
let rtc: poco.net.PocoPeerConnection;

const createConnection = async () => {
    socket = await poco.net.createPocoConnection({
        type: "socketIO",
        uri: "http://localhost:8080",
        localAddress,
    });

    socket.onEvent("peer connection setup", async ({ fromAddress, toAddress }: { fromAddress: string, toAddress: string }) => {
        history += `peer connection request from ${fromAddress}\n`

        if (toAddress != localAddress) {
            return;
        }

        remoteAddress = fromAddress;

        peer = await poco.net.createPocoPeerConnection({
            type: "socketIO",
            remoteAddress,
            connection: socket,
            timeout: 5000
        })

        peer.onMessage(async (payload: any) => {
            history += `${payload}\n`
        })

        await peer.connect();
    })

    socket.onEvent("peer connection established", async () => {
        history += `peer connection established\n`
    })

    await socket.connect();
}

const createPeerConnection = async () => {
    peer = await poco.net.createPocoPeerConnection({
        type: "socketIO",
        remoteAddress,
        connection: socket,
        timeout: 5000
    })

    peer.onMessage(async (payload: any) => {
        history += `${payload}\n`
    })

    peer.onEvent("webrtc offer", async (offer: RTCSessionDescriptionInit) => {
        rtc = new poco.net.PocoPeerWebRTCConnection(peer.remoteAddress, peer.localAddress, peer, {
            offer: offer
        });
    })

    await peer.connect();
}

const sendMessage = async () => {
    if (peer.status() != "connected") return;

    const messageToSend = message.trim();

    if (messageToSend.length === 0) return;

    peer.send(messageToSend);
}

const createWebRTCPeerConnection = async () => {
    rtc = await poco.net.createPocoPeerConnection({
        type: "webrtc",
        remoteAddress,
        connection: peer
    })

    await rtc.connect();
}

</script>

<template>
    <div class="p-8">
        <div class="flex h-12 items-center">
            <div>Local Address: </div>
            <input type="text" v-model="localAddress" class="border mx-2" />
            <button @click="createConnection"
                class="bg-purple-500 rounded-md px-4 text-white py-2 text-sm hover:bg-purple-300">connect</button>
        </div>
        <div class="flex h-12 items-center">
            <div>Remote Address: </div>
            <input type="text" v-model="remoteAddress" class="border mx-2" />
            <button @click="createPeerConnection"
                class="bg-purple-500 rounded-md px-4 text-white py-2 text-sm hover:bg-purple-300">connect</button>
        </div>
        <div class="flex h-12 items-center">
            <div class="pr-2">WebRTC Connection: </div>
            <button @click="createWebRTCPeerConnection"
                class="bg-purple-500 rounded-md px-4 text-white py-2 text-sm hover:bg-purple-300">connect</button>
        </div>
        <div class="flex h-12 items-center">
            <div>Message</div>
            <input type="text" v-model="message" class="border mx-2" />
            <button @click="sendMessage"
                class="bg-purple-500 rounded-md px-4 text-white py-2 text-sm hover:bg-purple-300">send</button>
        </div>
        <div class="flex flex-col justify-center">
            <div>History: </div>
            <textarea v-model="history" class="border w-72 h-64 mt-2"></textarea>
        </div>
    </div>
</template>