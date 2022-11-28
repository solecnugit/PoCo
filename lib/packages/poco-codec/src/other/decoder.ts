//这里是对VideoDecoder的封装，设计的原因是之前考虑decode和encode分开。
//目前 transcode 的video-transcoder worker和audio-transcoder worker能够直接处理转码需求
//这里目前没有作用，但是先保留等待日后重构代码。
export class WebVideoDecoder{
    private static videoDecoder: VideoDecoder;

    private constructor(init: VideoDecoderInit){
        WebVideoDecoder.videoDecoder = new VideoDecoder(init)
    }

    static getInstance(init: VideoDecoderInit) {
        if (!WebVideoDecoder.videoDecoder) {
             new WebVideoDecoder(init)
        }
        return WebVideoDecoder.videoDecoder;
    }

}

