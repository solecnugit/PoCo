export interface PocoMediaConnection {
    addTrack(track: MediaStreamTrack, ...streams: MediaStream[]): Promise<void>
    addTransceiver(trackOrKind: MediaStreamTrack | string, init?: any): void;
    onTrack(callback: (stream: readonly MediaStream[]) => Promise<void>): void;
}