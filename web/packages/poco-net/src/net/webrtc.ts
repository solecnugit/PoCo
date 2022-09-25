import { PocoConnection, PocoPeerConnection } from "./connection";
import { Address, PocoConnectionClosedError, PocoConnectionStatus, PocoPeerWebRTCConnectionOptions } from "./types";
import _ from "lodash";

export class PocoPeerWebRTCConnection extends PocoPeerConnection {
    protected rtcConnection: RTCPeerConnection;
    protected peerConnection: PocoConnection;

    protected options: Partial<PocoPeerWebRTCConnectionOptions>;

    constructor(localAddress: Address, remoteAddress: Address, peerConnection: PocoPeerConnection, opts?: Partial<PocoPeerWebRTCConnectionOptions>) {
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

        this.rtcConnection = new RTCPeerConnection(this.options.rtcConfiguration);
        this.rtcConnection.onicecandidate = (event) => {
            if (event.candidate) {
                this.peerConnection.emit("webrtc ice candidate", {
                    type: "new-ice-candidate",
                    candidate: event.candidate
                })
            }
        }

        this.rtcConnection.oniceconnectionstatechange = (event) => {
            switch (this.rtcConnection.iceConnectionState) {
                case "closed":
                case "failed":
                case "disconnected":
                    this.cleanup();
            }
        }

        this.rtcConnection.onsignalingstatechange = (event) => {
            switch (this.rtcConnection.signalingState) {
                case "closed":
                    this.cleanup();
            }
        }

        this.rtcConnection.onicegatheringstatechange = (event) => {

        }

        this.peerConnection = peerConnection;
        this.peerConnection.onEvent("webrtc offer", this.onOffer.bind(this));

        this.peerConnection.onEvent("webrtc answer", async (answer: RTCSessionDescriptionInit) => {
            const description = new RTCSessionDescription(answer);

            await this.rtcConnection.setRemoteDescription(description);
        })

        this.peerConnection.onEvent("webrtc ice candidate", async (event: { candidate: RTCIceCandidateInit, type: string }) => {
            const candidate = new RTCIceCandidate(event.candidate);

            await this.rtcConnection.addIceCandidate(candidate)
        })

        this.peerConnection.onEvent("webrtc close", async () => {
            this.cleanup();
        })

        if (this.options.offer) {
            this.onOffer(this.options.offer)
        }
    }

    async connect(): Promise<void> {
        if (this.peerConnection.status() === "pending") {
            await this.peerConnection.connect();
        }

        if (this.peerConnection.status() !== "connected") {
            throw new PocoConnectionClosedError(this.peerConnection);
        }

        const offer = await this.rtcConnection.createOffer(this.options?.rtcOfferOptions);

        await this.rtcConnection.setLocalDescription(offer);
        await this.peerConnection.emit("webrtc offer", {
            type: "video-offer",
            sdp: this.rtcConnection.localDescription
        });
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
        this.peerConnection.emit("webrtc close", {});

        this.cleanup();
    }

    async onOffer(offer: RTCSessionDescriptionInit) {
        const description = new RTCSessionDescription(offer);

        await this.rtcConnection.setRemoteDescription(description);

        const answer = await this.rtcConnection.createAnswer(this.options?.rtcAnswerOptions);

        await this.rtcConnection.setLocalDescription(answer);

        await this.peerConnection.emit("webrtc answer", {
            type: "video-answer",
            sdp: this.rtcConnection.localDescription
        });
    }

    status(): PocoConnectionStatus {
        throw new Error("Method not implemented.");
    }

    send<T>(payload: T): Promise<void> {
        throw new Error("Method not implemented.");
    }

    emit<T>(event: string, payload: T): Promise<void> {
        throw new Error("Method not implemented.");
    }

    onMessage<T>(callback: (payload: T) => Promise<void>): void {
        throw new Error("Method not implemented.");
    }

    onEvent<T>(event: string, callback: (payload: T) => Promise<void>, once?: boolean | undefined): void {
        throw new Error("Method not implemented.");
    }
}