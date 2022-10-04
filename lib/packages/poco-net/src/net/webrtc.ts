import { PocoPeerConnection } from "./connection";
import { Address, ChannelId, PocoPeerWebRTCConnectionOptions } from "./types";
import { PocoMediaConnection } from "./media";
import _ from "lodash";
import { PocoConnectionClosedError } from "./error";
import { deserializeMessagePayload, serializePocoMessagePayload } from "../protocol";
import { DefaultEventsMap, EventsMap, ReservedOrUserEventNames, ReservedOrUserEventParameters } from "poco-util";
import { PocoPeerSocketIOConnectionEvents } from "./socketIO";

export type PocoPeerWebRTCInternalChannel = "message" | "event";

export type PocoPeerWebRTCConnectionEvents = {
    "webrtc offer": (this: ThisType<PocoPeerWebRTCConnection>, offer: RTCSessionDescriptionInit) => void;
    "webrtc answer": (this: ThisType<PocoPeerWebRTCConnection>, answer: RTCSessionDescriptionInit) => void;
    "webrtc candidate": (this: ThisType<PocoPeerWebRTCConnection>, candidate: RTCIceCandidateInit) => void;
    "webrtc destroy": (this: ThisType<PocoPeerWebRTCConnection>) => void;
}

export type PocoPeerWebRTCConnectionInternalEvents = {
    "channel open": (this: ThisType<PocoPeerWebRTCConnection>, channel: RTCDataChannel) => void;
    "channel close": (this: ThisType<PocoPeerWebRTCConnection>, channel: RTCDataChannel) => void;
    "channel error": (this: ThisType<PocoPeerWebRTCConnection>, channel: RTCDataChannel) => void;
}

export class PocoPeerWebRTCConnection<Events extends EventsMap = DefaultEventsMap>
    extends PocoPeerConnection<Events, PocoPeerWebRTCConnectionInternalEvents>
    implements PocoMediaConnection {

    protected rtcConnection: RTCPeerConnection;
    protected peerConnection: PocoPeerConnection<PocoPeerWebRTCConnectionEvents>;

    protected options: Partial<PocoPeerWebRTCConnectionOptions>;
    protected channels: Map<ChannelId, RTCDataChannel>;

    constructor(
        localAddress: Address,
        remoteAddress: Address,
        peerConnection: PocoPeerConnection<PocoPeerWebRTCConnectionEvents>,
        opts?: Partial<PocoPeerWebRTCConnectionOptions>
    ) {

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
                this.peerConnection.send("webrtc candidate", candidate)
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

            if (channelId === "poco") {
                this.setupInternalChannel();
            }
        })

        this.peerConnection = peerConnection;

        this.peerConnection.on("webrtc answer", async (answer) => {
            const description = new RTCSessionDescription(answer);

            await this.rtcConnection.setRemoteDescription(description);
        })

        this.peerConnection.on("webrtc candidate", async (candidate) => {
            const iceCandidate = new RTCIceCandidate(candidate);

            await this.rtcConnection.addIceCandidate(iceCandidate)
        })

        this.peerConnection.on("webrtc destroy", () => {
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
            this.setupInternalChannel();

            const offer = await this.rtcConnection.createOffer(this.options?.rtcOfferOptions);

            await this.rtcConnection.setLocalDescription(offer);

            this.peerConnection.send("webrtc offer",
                this.rtcConnection.localDescription!
            );
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
        this.peerConnection.send("webrtc destroy");

        this.cleanup();
    }

    private async setupOffer(offer: RTCSessionDescriptionInit) {
        const description = new RTCSessionDescription(offer);

        await this.rtcConnection.setRemoteDescription(description);

        const answer = await this.rtcConnection.createAnswer(this.options?.rtcAnswerOptions);

        await this.rtcConnection.setLocalDescription(answer);

        this.peerConnection.send("webrtc answer",
            this.rtcConnection.localDescription!
        );
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

    getChannel(id: ChannelId, opts?: RTCDataChannelInit): RTCDataChannel {
        let channel = this.channels.get(id);

        if (!channel) {
            channel = this.rtcConnection.createDataChannel(id, opts);

            this.channels.set(id, channel)
        }

        return channel;
    }

    protected setupInternalChannel() {
        const channel = this.getChannel("poco");

        channel.addEventListener("open", () => {
            this.triggerEvent("channel open", channel)
        })

        channel.addEventListener("close", () => {
            this.triggerEvent("channel close", channel)
        })

        channel.addEventListener("error", () => {
            this.triggerEvent("channel error", channel)
        })

        channel.addEventListener("message", ({ data }) => {
            const [event, ...payload] = deserializeMessagePayload(data);

            this.triggerEvent(event, ...payload);
        })
    }

    send<Event extends ReservedOrUserEventNames<PocoPeerWebRTCConnectionInternalEvents, Events>>(type: Event, ...payload: ReservedOrUserEventParameters<PocoPeerSocketIOConnectionEvents, Events, Event>): void | Promise<void> {
        const channel = this.getChannel("poco");

        channel.send(serializePocoMessagePayload([
            type,
            ...payload
        ]))
    }

    // send(payload: PocoMessagePayload): void | Promise<void> {
    //     const channel = this.getChannel("message");

    //     channel.send(serializePocoMessagePayload(payload));
    // }

    // override emit<Event extends EventNames<Events & PocoConnectionEvents>, Payload extends Parameters<OmitThisParameter<(Events & PocoConnectionEvents)[EventNames<Events & PocoConnectionEvents>]>> = Parameters<OmitThisParameter<(Events & PocoConnectionEvents)[Event]>>>(event: Event, payload: Payload): void {
    //     const channel = this.getChannel("event");

    //     channel.send(serializePocoMessagePayload([
    //         event,
    //         ...payload
    //     ]))
    // }
}