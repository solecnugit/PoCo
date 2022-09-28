export interface PocoMediaConnection {
    addTrack(track: MediaStreamTrack, ...streams: MediaStream[]): Promise<void>
    addTransceiver(trackOrKind: MediaStreamTrack | string, init?: any): void;
    onTrack(callback: (this: PocoMediaConnection, event: RTCTrackEvent) => Promise<void>): void;
}