//这里是对VideoEncoder的封装，设计的原因是之前考虑decode和encode分开。
//目前 transcode 的video-transcoder worker和audio-transcoder worker能够直接处理转码需求
//这里目前没有作用，但是先保留等待日后重构代码。
export class WebVideoEncoder{
    private static videoEncoder: VideoEncoder;

    private constructor(init: VideoEncoderInit){
        WebVideoEncoder.videoEncoder = new VideoEncoder(init)
    }

    static getInstance(init: VideoEncoderInit) {
        if (!WebVideoEncoder.videoEncoder) {
             new WebVideoEncoder(init)
        }
        return WebVideoEncoder.videoEncoder;
    }

}