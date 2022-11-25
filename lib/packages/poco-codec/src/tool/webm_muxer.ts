
//这里分别声明了VIDEO_STREAM_TYPE和AUDIO_STREAM_TYPE的两种类型
// const VIDEO_STREAM_TYPE = 1;
// const AUDIO_STREAM_TYPE = 0;

import {max_video_config} from './resolution'
export class WebmMuxer{
    constructor(){

    }

    vp9_encoder_constraints = {
        // codec: 'av01.0.08M.08',
        codec: 'vp09.00.10.08.01',
        width: 720,
        height: 1280,
        bitrate: 1725000,
        framerate: 30,
        latencyMode: 'realtime'
    }

    av1_encoder_constraints = {
      
      codec: 'av01.0.00M.08',
      width: 720,
      height: 1280,
      bitrate: 1725000,
      framerate: 30,
      latencyMode: 'realtime'
    }
    async initialize() {
        // if(demuxer.streamType === AUDIO_STREAM_TYPE) {

        // } else {
        //     // this.codec = 'av01.0.00M.08',//这里先写死
        //     this.codec = 'vp09.00.10.08.01'
        //     // this.displayWidth = demuxer.getDecoderConfig().displayWidth;
        //     // this.displayHeight = demuxer.getDecoderConfig().displayHeight;
        //     this.width= 640,
        //     this.height= 360,
        //     this.bitrate = 2500 * 100;
        //     this.framerate = 30;
        //     this.latencyMode = 'realtime';
        // }
        
    
        // //不管是videotrack还是audiotrack都ready了
        // await this._tracksReady();
    
        // if (this.streamType == AUDIO_STREAM_TYPE) {
        //   this._selectTrack(this.audioTrack);
        // } else {
        //   this._selectTrack(this.videoTrack);
        // }
        // console.log('muxer initialize finished')
      }

      //目前的getEncoderConfig应当返回视频的encoderconfig
      //对于音频的encoderconfig，需要做的事情现在在audio_decoder中完成，后续仍然需要迭代。
    async getEncoderConfig(decodeconfig: VideoDecoderConfig, bitrate: number, framerate: number) {

      this.vp9_encoder_constraints.width = decodeconfig.codedWidth!;
      this.vp9_encoder_constraints.height = decodeconfig.codedHeight!;


      this.vp9_encoder_constraints.bitrate = bitrate;
      this.vp9_encoder_constraints.framerate = framerate;

          console.log('in getencoder config');
          console.log(this.vp9_encoder_constraints)
            return await max_video_config({
                ...this.vp9_encoder_constraints,
                ratio: this.vp9_encoder_constraints.width / this.vp9_encoder_constraints.height
            }) || await max_video_config(this.vp9_encoder_constraints);
      }
}