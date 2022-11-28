// import { AUDIO_STREAM_TYPE } from "./pull_demuxer_base.js";
// import { RingBuffer } from "../third_party/ringbufjs/ringbuf.js";

// const DATA_BUFFER_DECODE_TARGET_DURATION = 0.3;
// const DATA_BUFFER_DURATION = 0.6;
// const DECODER_QUEUE_SIZE_MAX = 5;

import { MP4PullDemuxer } from '../tool/mp4_demuxer';
import { WebmMuxer } from '../tool/webm_muxer';
import { SampleLock } from '../tool/SampleLock'
import  {AUDIO_STREAM_TYPE, ENCODER_QUEUE_SIZE_MAX, debugLog} from '../tool/type'

//importScripts在ts环境中不适用，因此先注释，看看会出什么问题再解决
// self.importScripts('../external-js/mp4box.all.min.js');


var frameCount = 0;
var chunkCount = 0;
var rechunkCount = 0;

let audioTranscoder: AudioTranscoder|undefined;

//这里整一块都是audiotranscoder module的回调部分，其实仅仅监听两个方法。
onmessage = async function (e) {
  const msg = e.data;
  console.log('in audio data message...')
  if(!audioTranscoder)
    audioTranscoder = new AudioTranscoder();
  switch (msg.type) {
    //监听方法1： initialize，将会初始化audioTranscoder。
    case 'initialize':
      console.log('audio transcoder: case initialize is triggered');
      // let demuxer = await import('../tool/mp4_demuxer');
      let audioDemuxer =  new MP4PullDemuxer();
      // console.log('finish audioDemuxer');
      let muxer = new WebmMuxer();
      //这里可能要重写
      //将提取出几个config的方法单独挪出来，直接将config传入initialize
      console.log('audio_worker: waiting for encodeconfig')
      const encodeconfig = await audioTranscoder.initialize(audioDemuxer, muxer, msg.buffer);
      console.log('audio_worker: getting encodeconfig')
      console.log("audio transcoder: audioTranscoder initialize finished");
      console.log('initialize done');
      this.self.postMessage({
        type: 'initialize-done',
        workerType : 'audio',
        config: {
          bit_depth: 0,
          sample_rate: encodeconfig.sampleRate,
          channels: encodeconfig.numberOfChannels,
          codec_id: 'A_OPUS'
        }
      });
      break;
    //监听方法2： start-transcode：将会开始音频的转码。
    case 'start-transcode':
      //初始调用fillFrameBuffer
      // console.log('audio: transcoder is below')
      // console.log(audioTranscoder.encoder);
      // console.log(audioTranscoder.decoder);
      // console.log('audio: transcoder: case start-transcode is triggered');
      audioTranscoder.fillDataBuffer()
      break;
  }
}


class AudioTranscoder {
  //防止并发访问公共资源做的锁的设计，当其为true时，不能够访问。
  fillInProgress: boolean = false;
  //同样是锁，避免对公共资源的并发访问。
  lock: SampleLock | undefined;

  demuxer: MP4PullDemuxer|undefined;
  
  encoder: AudioEncoder|undefined;
  
  decoder: AudioDecoder|undefined;
  
  //判断audio的解码是否完成，如果完成的话，overaudio将为true。
  overaudio: boolean = false;

  //重要参数：samplerate
  sampleRate: number = 0;

  //重要参数：channelCount
  channelCount: number = 0;

  //webmmuxer，主要用于获取mux的相关信息（目前没有作用，因为encoder的config设置都是在transcoder中完成了）
  muxer: WebmMuxer|undefined;

  //rest_number：默认为-1，当getNextChunk无法返回samples时，将会返回它，用于判断音频是否解码完成。
  rest_number: number = -1;

  //exited：用于标识音频已经解码完成，防止多次发送exit信号，导致转码异常。
  exited: boolean = false;

  async initialize(demuxer: MP4PullDemuxer, muxer: WebmMuxer, buffer: ArrayBuffer) {
    // console.log('into audiotranscoder init')
    this.fillInProgress = false;
    this.lock = new SampleLock();
    this.demuxer = demuxer;
    this.muxer = muxer;
    this.overaudio = false;

    // console.log('audiotranscoder ready for initialize demuxer')
    await this.demuxer.initialize(AUDIO_STREAM_TYPE, buffer);
    console.log('audiotranscoder finish initialize demuxer')


    this.decoder = new AudioDecoder({
      output: this.bufferAudioData.bind(this),
      error: e => console.error(e)
    });
    // console.log('before audio decode config')
    const decodeconfig: AudioEncoderConfig = <AudioEncoderConfig>this.demuxer.getDecoderConfig();
    // console.log('audio decodeconfig');
    // console.log(decodeconfig)
    //从decoder获得的sampleRate以及numberOfChannels直接赋给了this
    this.sampleRate = decodeconfig.sampleRate;
    this.channelCount = decodeconfig.numberOfChannels;

    console.log('audio decoder below');
    console.log(this.decoder)
    // debugLog(decodeconfig);

    console.assert(AudioDecoder.isConfigSupported(decodeconfig));
    this.decoder.configure(decodeconfig);

    //encoder读取audio data并且将其再次encode
    this.encoder = new AudioEncoder({
      output: this.consumeAudioData.bind(this),
      error: e => console.error(e)
    })
    //当转为webm格式时，音频的config直接写死
    //目前的opus的encodeconfig不经过webm-muxer，直接从原来的audiodecoder生成而来
    const encodeconfig = {
      codec: 'opus',
      bitrate: 128 * 1000,
      sampleRate: this.sampleRate,
      numberOfChannels: this.channelCount
    }
    console.assert(AudioEncoder.isConfigSupported(encodeconfig));
    this.encoder.configure(encodeconfig);
    return encodeconfig;
  }



  // 作用是确保只有一个能进入这个过程
  async fillDataBuffer() {

    if(this.audioDataFull()){
      console.log('audio data full');
      return;
    }
    // This method is called from multiple places to ensure the buffer stays
    // healthy. Sometimes these calls may overlap, but at any given point only
    // one call is desired.
    if (this.fillInProgress)
      return;

    this.fillInProgress = true;
    // This should be this file's ONLY call to the *Internal() variant of this method.
    // await this.fillDataBufferInternal();
    
    while (this.decoder!.decodeQueueSize < ENCODER_QUEUE_SIZE_MAX && 
      //返回队列中挂起的解码请求数。
      this.encoder!.encodeQueueSize < ENCODER_QUEUE_SIZE_MAX && !this.overaudio) {
        let chunk = await this.demuxer!.getNextChunk();

      //如果chunk为number，那么就代表所有的samples都完成了
      //这个时候获得rest_number
        if(typeof chunk === 'number'){
          this.overaudio = true; 
          this.rest_number = chunk;
          console.log('get audio rest_number'+ this.rest_number)
          this.decoder!.flush();
          this.encoder!.flush();
        }
        else{ 
          chunkCount++;
          // console.log('audio chunk  count');
          // console.log(chunkCount);
          this.decoder!.decode(chunk);
        }
      }
    // this.fillInProgress = false;
    this.fillInProgress = false;

    if(!this.overaudio && this.encoder!.encodeQueueSize === 0)
      setTimeout(this.fillDataBuffer.bind(this), 0);

  }

  //判断audio解码过程是否full
  audioDataFull(){
    return this.encoder!.encodeQueueSize >= ENCODER_QUEUE_SIZE_MAX;
  }


  //这一步是audioDecoder的回调，通过观察控制台输出结果，可以确定的是audio data 和 getNextChunk得到的chunk是一一对应的。
  bufferAudioData(frame: AudioData) {
    frameCount++;
        //暂时去掉
    // console.log('audio data count');
    // console.log(framecount);

    // console.log('audio frame')
    // console.log(frame)
    
    debugLog(`bufferFrame(${frame.timestamp})`);
    // frameCount ++;
    // console.log('audio framecount')
    // console.log(frameCount);
    this.encoder!.encode(frame);
    //这里注释了，为了暂停bufferframe
    // this.fillFrameBuffer();
    frame.close();
    // this.frameBuffer.push(frame);
  }



  //这是自己写的encoder的回调，完成encode的过程后会自动给主线程发送信息
  async consumeAudioData(chunk: EncodedAudioChunk) {

    const data = new ArrayBuffer(chunk.byteLength);
    chunk.copyTo(data);
    self.postMessage({
      type: 'audio-data',
      timestamp: chunk.timestamp,
      duration: chunk.duration,
      is_key: true,
      data
        //@ts-ignore
    }, [data])

    //请求锁
    await this.lock!.status;
    this.lock!.lock();
    rechunkCount++;
    this.lock!.unlock();

        //暂时去掉
        // console.log('audio rechunk count');
        // console.log(rechunkCount)

    if(!this.overaudio && this.encoder!.encodeQueueSize === 0)
        this.fillDataBuffer();
    if(this.overaudio && this.encoder!.encodeQueueSize === 0 && this.decoder!.decodeQueueSize === 0){
      //判断已经解码完成
      //条件1：frmeCount === chunkCount
      //条件2：exited为false
      //条件3：当前chunkCount对1000的余数为rest_number
      if(frameCount === chunkCount && chunkCount % 1000 === this.rest_number && !this.exited){
        this.exited = !this.exited;
        self.postMessage({type: 'exit'})
        console.log('post audio transcoder exit message to self...')
        console.log('current audio framecount'+ frameCount)
        console.log('current audio chunkCount'+ chunkCount)
      }
    }
  }

}
