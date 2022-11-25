// import { VIDEO_STREAM_TYPE } from "./pull_demuxer_base.js";
// import { MP4PullDemuxer } from "../mp4_pull_demuxer.js";
// import { max_video_config } from "./resolution";

import { MP4PullDemuxer } from '../tool/mp4_demuxer';
import { WebmMuxer } from '../tool/webm_muxer';
import { SampleLock } from '../tool/SampleLock'

import { DECODER_QUEUE_SIZE_MAX, VIDEO_STREAM_TYPE, debugLog } from '../tool/type'

//importScripts在ts环境中不适用，因此先注释，看看会出什么问题再解决
// self.importScripts('../external-js/mp4box.all.min.js');

// const VIDEO_STREAM_TYPE = 1;
// const AUDIO_STREAM_TYPE = 0;
// const DECODER_QUEUE_SIZE_MAX = 5;
// const ENABLE_DEBUG_LOGGING = false;
var framecount = 0;
var chunkCount = 0;
var midCount = 0;

let videoTranscoder: VideoTranscoder|null = null;



const vp9_params = {
  profile: 0,
  level: 10,
  bit_depth: 8,
  // chroma_subsampling: chroma_el.value ? 2 : 1
  chroma_subsampling: 1
};

const av1_params = {
  profile: 0,
  level: 0,
  tier: 0,
  high_bitdepth: false,
  twelve_bit: false,
  monochrome: false,
  chroma_subsampling_x: false,
  chroma_subsampling_y: false,
  chroma_sample_position: 0,
};

onmessage = async function (e) {
  const msg = e.data;
  if(videoTranscoder === null)
    videoTranscoder = new VideoTranscoder();
  switch (msg.type) {
    case 'initialize':
      console.log('video transcoder: case initialize is triggered');
      let videoDemuxer =  new MP4PullDemuxer();
      // console.log('finish videoDemuxer')
      let muxer = new WebmMuxer();
      // console.log('finish videoWebmMuxer')
      //这里可能要重写
      //将提取出几个config的方法单独挪出来，直接将config传入initialize
      const encodeconfig = await videoTranscoder.initialize(videoDemuxer, muxer, msg.buffer);
      // console.log("video transcoder: Transcoder initialize finished");
      // console.log('video transcoder: initialize done');
      this.self.postMessage({
        type: 'initialize-done',
        workerType : 'video',
        config: {
          width: encodeconfig?.width,
          height: encodeconfig?.height,
          frame_rate: encodeconfig?.framerate,
          // codec_id: encodeconfig.codec,
          codec_id: 'V_VP9',
          ...vp9_params
        }
      });
      break;
    case 'start-transcode':
      //初始调用fillFrameBuffer
      console.log('video transcoder is below')
      console.log(videoTranscoder.encoder);
      console.log(videoTranscoder.decoder);
      console.log('video transcoder: case start-transcode is triggered');
      videoTranscoder.fillFrameBuffer()
      break;
  }
}




// Controls demuxing and decoding of the video track, as well as rendering
// VideoFrames to canvas. Maintains a buffer of DECODER_QUEUE_SIZE_MAX
// decoded frames for future rendering.
//控制了解复用和对视频轨道的解码
class VideoTranscoder {
  encoder: VideoEncoder|undefined;
  decoder: VideoDecoder|undefined;
  lock: SampleLock|undefined;
  over: boolean = false;
  demuxer: MP4PullDemuxer|undefined;
  muxer: WebmMuxer|undefined;
  fillInProgress: boolean = false;
  rest_number: number = -1;
  
  async initialize(demuxer: MP4PullDemuxer, muxer: WebmMuxer, buffer: ArrayBuffer) {
    // console.log('into videotranscoder init')
    //frameBuffer其实也已经没有用了，这里注释
    // this.frameBuffer = [];
    //是否在fillinprogress，默认是false
    this.fillInProgress = false;
    // this.addProcess = false;

    this.demuxer = demuxer;
    this.muxer = muxer;
    this.over = false;

    this.lock = new SampleLock();
    //根据VIDEO_STREAM_TYPE进行初始化，这里进行了demuxer的初始化
    
    await this.demuxer?.initialize(VIDEO_STREAM_TYPE, buffer);
    console.log('videotranscoder finish initialize demuxer')

    const decodeconfig = this.demuxer?.getDecoderConfig();
    const otherconfig = this.demuxer?.getOtherVideoConfig();
    console.log('see see decodeconfig')
        console.log(decodeconfig);
    const encodeconfig = await this.muxer?.getEncoderConfig(decodeconfig, otherconfig.bitrate, Number(otherconfig.framerate));

    console.log('video encodeconfig');
    console.log(encodeconfig)

    this.decoder = new VideoDecoder({
      //每进来一个frame，将其缓存进frameBuffer中
      output: this.bufferFrame.bind(this),
      error: e => console.error(e),
    });

    console.assert(VideoDecoder.isConfigSupported(decodeconfig))
    this.decoder.configure(decodeconfig);
   
    //初始化encoder
    this.encoder = new VideoEncoder({
      output: this.consumeFrame.bind(this),
      error: e => console.error(e)
    })
    console.log('encoder is below')
    console.log(this.encoder)
    console.assert(VideoEncoder.isConfigSupported(<VideoEncoderConfig>encodeconfig))
    this.encoder.configure(<VideoEncoderConfig>encodeconfig);
    // console.log("decoder & encoder configured finished")
    //要将相关参数返回去，这里return
    return encodeconfig;
  }



  //填充framebuffer
  async fillFrameBuffer() {
    if (this.frameBufferFull()) {
      console.log('video frame buffer full');

      setTimeout(this.fillFrameBuffer.bind(this), 20);
    }
    

    // This method can be called from multiple places and we some may already
    // be awaiting a demuxer read (only one read allowed at a time).
    //这个方法可以从多个地方调用，有时可能已经在等待demuxer读取（一次只允许一个读取）。
    //fillinprogress是控制并发的
    if (this.fillInProgress) {
      return ;
    }
    this.fillInProgress = true;

    //当已经buffer的frame和decoded序列长度都小于DECODER_QUEUE_SIZE_MAX（3）时，就会进行getNextChunk，并且decode
    while ((<number>(this.decoder?.decodeQueueSize) < DECODER_QUEUE_SIZE_MAX) && 
      //返回队列中挂起的解码请求数。
        (<number>(this.encoder?.encodeQueueSize) < DECODER_QUEUE_SIZE_MAX) && !this.over) {
          
      let chunk = await this.demuxer?.getNextChunk();

      // console.log('get chunk')
      // console.log(chunk);
      if(typeof chunk === 'number'){
        this.rest_number = chunk;
        this.over = true; 
        this.decoder!.flush();
      }
      else{ 
        chunkCount++;
        console.log('video chunk count')
        console.log(chunkCount)
        this.decoder!.decode(chunk!);
      }
    }
    this.fillInProgress = false;

    

    // Give decoder a chance to work, see if we saturated the pipeline.
    //当encoder的encodeQueueSize为0时才会再次开始编码
    if(!this.over && this.encoder?.encodeQueueSize === 0)
      setTimeout(this.fillFrameBuffer.bind(this), 0);
  }

  //判断frame是否满
  frameBufferFull() {
    return ((<number>this.encoder?.encodeQueueSize) >= DECODER_QUEUE_SIZE_MAX);
  }

  //将frame buffer起来
  bufferFrame(frame: VideoFrame) {
    midCount++;
    console.log('video midcount'+midCount)
    debugLog(`bufferFrame(${frame.timestamp})`);
    this.encoder?.encode(frame);
    //这里注释了，为了暂停bufferframe
    // this.fillFrameBuffer();
    frame.close();
    // this.frameBuffer.push(frame);
  }

  //encoder的回调函数
  async consumeFrame(chunk: EncodedVideoChunk) {
    //这个chunk的duration属性为0，但是也许可以通过timestamp计算出来？不知道会不会有影响？
    // console.log(chunk);
    const data = new ArrayBuffer(chunk.byteLength);
    chunk.copyTo(data);
    self.postMessage({
      //这里要注意，后面会用type来替代
      type: 'video-data',
      timestamp: chunk.timestamp,
      duration: chunk.duration,
      is_key: chunk.type === 'key',
      data
      //@ts-ignore
    }, [data]);

    
    await this.lock?.status;
    this.lock?.lock();
    framecount++;
    this.lock?.unlock();

    console.log('video framecount')
    console.log(framecount);
    
    //调用的主要地方，consumeFrame处
    if(!this.over && this.encoder?.encodeQueueSize === 0)
        this.fillFrameBuffer();
    
    if(this.encoder?.encodeQueueSize === 0 && this.decoder?.decodeQueueSize === 0 && this.over){
      console.log(framecount)
      console.log('video framecount');
      console.log(chunkCount);
      console.log('video chunkCount');
      //根据bbb视频多次实验，发现h264格式下解码出来的帧数总是会比总帧数少1
      //发现hevc格式下解码出来的帧数会比总帧数少3
      //切换试验视频，framecount比chunkCount少4
      if(framecount === chunkCount){
        console.log('current video')
        console.log(framecount)
        console.log(chunkCount)
        console.log('post videotranscoder exit message to self...')
        console.log(framecount)
        self.postMessage({type: 'exit'})
      }
    }
  }

}
