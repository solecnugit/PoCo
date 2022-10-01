import { PocoConnectionEvents, PocoPeerConnection } from "./connection";
import { Address, ChannelId, PocoPeerWebRTCConnectionOptions } from "./types";
import { PocoMediaConnection } from "./media";
import _ from "lodash";
import { PocoConnectionClosedError } from "./error";
import { deserializePocoObject, PocoObject, serializePocoObject } from "../protocol";
import { DefaultEventsMap, EventNames, EventParameter, EventParameters, EventsMap } from "./event";
import { Expand } from "../util";

export type PocoPeerWebRTCInternalChannel = "message" | "event";

export interface PocoPeerWebRTCConnectionEvents {
    "webrtc offer": (this: PocoPeerWebRTCConnection, args: { offer: RTCSessionDescriptionInit }) => void;
    "webrtc answer": (this: PocoPeerWebRTCConnection, args: { answer: RTCSessionDescriptionInit }) => void;
    "webrtc candidate": (this: PocoPeerWebRTCConnection, args: { candidate: RTCIceCandidateInit }) => void;
    "webrtc destroy": (this: PocoPeerWebRTCConnection, args: {}) => void;
}

export class PocoPeerWebRTCConnection<Events extends EventsMap = DefaultEventsMap>
    extends PocoPeerConnection<Events>

    implements PocoMediaConnection {

    protected rtcConnection: RTCPeerConnection;
    protected peerConnection: PocoPeerConnection<PocoPeerWebRTCConnectionEvents>;

    protected options: Partial<PocoPeerWebRTCConnectionOptions>;
    protected channels: Map<ChannelId, RTCDataChannel>;

    constructor(localAddress: Address,
        remoteAddress: Address,
        peerConnection: PocoPeerConnection<PocoPeerWebRTCConnectionEvents>,
        opts?: Partial<PocoPeerWebRTCConnectionOptions>) {

        super("webrtc", localAddress, remoteAddress)

        this.options = _.defaults(opts, {
            rtcConfiguration: {
                iceServers: [
                    {
                        urls: "stun:stun.l.google.com:19302"
                    }
                ]
            }
        });

        this.channels = new Map();

        this.rtcConnection = new RTCPeerConnection(this.options.rtcConfiguration);
        this.rtcConnection.addEventListener("icecandidate", ({ candidate }) => {
            if (candidate && this.rtcConnection.remoteDescription) {
                this.peerConnection.emit("webrtc candidate", { candidate })
            }
        })

        this.rtcConnection.addEventListener("iceconnectionstatechange", () => {
            this.setStatus(this.rtcConnection.connectionState)

            switch (this.rtcConnection.iceConnectionState) {
                case "closed":
                case "failed":
                case "disconnected":
                    this.cleanup();
            }
        });

        this.rtcConnection.addEventListener("signalingstatechange", () => {
            switch (this.rtcConnection.signalingState) {
                case "closed":
                    this.cleanup();
            }
        })

        this.rtcConnection.addEventListener("icegatheringstatechange", () => {

        })

        this.rtcConnection.addEventListener("connectionstatechange", () => {
            const stage = this.rtcConnection.connectionState;

            this.setStatus(stage)
        })

        this.rtcConnection.addEventListener("datachannel", ({ channel }) => {
            const channelId = channel.label;

            this.channels.set(channelId, channel)

            if (channelId === "message") {
                this.setupMessageChannel();
            } else if (channelId === "event") {
                this.setupEventChannel();
            }
        })

        this.peerConnection = peerConnection;

        this.peerConnection.once("webrtc answer", async ({ answer }) => {
            const description = new RTCSessionDescription(answer);

            await this.rtcConnection.setRemoteDescription(description);
        })

        this.peerConnection.on("webrtc candidate", async ({ candidate }) => {
            const iceCandidate = new RTCIceCandidate(candidate);

            await this.rtcConnection.addIceCandidate(iceCandidate)
        })

        this.peerConnection.once("webrtc destroy", async () => {
            this.cleanup();
        })
    }

    async connect(): Promise<void> {
        if (this.peerConnection.status() === "new") {
            await this.peerConnection.connect();
        }

        if (this.peerConnection.status() !== "connected") {
            throw new PocoConnectionClosedError(this.peerConnection);
        }

        if (this.options.offer) {
            this.setupOffer(this.options.offer)
        } else {
            this.setupInternalChannels();

            const offer = await this.rtcConnection.createOffer(this.options?.rtcOfferOptions);

            await this.rtcConnection.setLocalDescription(offer);
            await this.peerConnection.emit("webrtc offer", {
                offer: this.rtcConnection.localDescription!
            });
        }
    }

    cleanup() {
        this.rtcConnection.ontrack = null;
        this.rtcConnection.onicecandidate = null;
        this.rtcConnection.oniceconnectionstatechange = null;
        this.rtcConnection.onicegatheringstatechange = null;
        this.rtcConnection.onsignalingstatechange = null;
        this.rtcConnection.onnegotiationneeded = null;

        this.rtcConnection.close()
    }

    async disconnect(): Promise<void> {
        this.peerConnection.emit("webrtc destroy", {});

        this.cleanup();
    }

    private async setupOffer(offer: RTCSessionDescriptionInit) {
        const description = new RTCSessionDescription(offer);

        await this.rtcConnection.setRemoteDescription(description);

        const answer = await this.rtcConnection.createAnswer(this.options?.rtcAnswerOptions);

        await this.rtcConnection.setLocalDescription(answer);

        await this.peerConnection.emit("webrtc answer", {
            answer: this.rtcConnection.localDescription!
        });
    }

    getRTCConnection(): RTCPeerConnection {
        return this.rtcConnection;
    }

    addTrack(track: MediaStreamTrack, ...streams: MediaStream[]) {
        this.rtcConnection.addTrack(track, ...streams);
    }

    onTrack(callback: (event: RTCTrackEvent) => void): void {
        this.rtcConnection.addEventListener("track", callback)
    }

    addTransceiver(trackOrKind: MediaStreamTrack | string, init?: RTCRtpTransceiverInit): void {
        this.rtcConnection.addTransceiver(trackOrKind, init)
    }

    // status(): PocoConnectionStatus {
    //     return this.rtcConnection.connectionState;
    // }

    getChannel(id: ChannelId, opts?: RTCDataChannelInit): RTCDataChannel {
        let channel = this.channels.get(id);

        if (!channel) {
            channel = this.rtcConnection.createDataChannel(id, opts);

            this.channels.set(id, channel)
        }

        return channel;
    }

    protected setupInternalChannel(channel: RTCDataChannel) {
        channel.addEventListener("open", () => {
            this.triggerEvent("channel open", { channel: channel })
        })

        channel.addEventListener("close", () => {
            this.triggerEvent("channel close", { channel: channel })
        })

        channel.addEventListener("error", (event) => {
            this.triggerEvent("channel error", { channel: channel, event })
        })
    }

    protected setupMessageChannel() {
        const messageChannel = this.getChannel("message");

        this.setupInternalChannel(messageChannel);

        messageChannel.addEventListener("message", ({ data }) => {
            const message = deserializePocoObject(data);

            this.onMessage(message);
        })
    }

    protected setupEventChannel(): void {
        const eventChannel = this.getChannel("event");

        this.setupInternalChannel(eventChannel);

        eventChannel.addEventListener("message", ({ data }) => {
            const { event: eventName, payload } = deserializePocoObject(data) as {
                event: EventNames<Events>,
                payload: EventParameters<Events>
            };

            this.triggerEvent(eventName, payload);
        })
    }

    protected setupInternalChannels(): void {
        this.setupEventChannel();
        this.setupMessageChannel();
    }

    send(payload: PocoObject): void | Promise<void> {
        const channel = this.getChannel("message");

        channel.send(serializePocoObject({
            message: payload
        }));
    }

    emit<Event extends EventNames<Events & PocoConnectionEvents>, Payload extends EventParameter<Events & PocoConnectionEvents, Event> = EventParameter<Events & PocoConnectionEvents, Event>>
        (event: Event, payload: Payload): void | Promise<void> {
        const channel = this.getChannel("event");

        channel.send(serializePocoObject({
            event: event as string,
            payload: payload as any
        }))
    }

    onMessage(message: PocoObject): void | Promise<void> {
        this.triggerEvent("message", { message });
    }
}