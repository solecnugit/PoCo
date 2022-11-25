import { MP4File, createFile, MP4Sample, MP4Info, MP4VideoTrack, MP4ArrayBuffer, MP4AudioTrack, DataStream} from "mp4box"
import {  AUDIO_STREAM_TYPE, ENABLE_DEBUG_LOGGING } from "./type";


//创建基类PullDemuxerBase
class PullDemuxerBase {
  
  // Starts fetching file. Resolves when enough of the file is fetched/parsed to
  // populate getDecoderConfig().
  //@ts-ignore
  async initialize(streamType: number, buffer: ArrayBuffer) {}

  // Returns either an AudioDecoderConfig or VideoDecoderConfig based on the
  // streamType passed to initialize().
  getDecoderConfig() {}

  // Returns either EncodedAudioChunks or EncodedVideoChunks based on the
  // streamType passed to initialize(). Returns null after EOF.
  async getNextChunk(): Promise<any> {}
}

export class MP4PullDemuxer extends PullDemuxerBase {
  // fileUri: string;
  source: MP4Source|undefined;
  readySamples: MP4Sample[] | undefined;
  over: boolean = false;
  _pending_read_resolver: undefined | ((current: MP4Sample| undefined | null)=>void) | null;
  streamType: number = 0;
  audioTrack: MP4AudioTrack|undefined;
  videoTrack: MP4VideoTrack|undefined;
  selectedTrack: any;
  rest_number: number = 0;
  constructor() {
    super();
    // this.fileUri = fileUri;
  }

  override async initialize(streamType: number, buffer: ArrayBuffer) {
    
    // console.log('this.fileUri')
    // console.log(this.fileUri)
    this.source = new MP4Source(buffer);
    // console.log(streamType);
    // console.log(this.fileUri+'finish init source')

    this.readySamples = [];
    this.over = false;
    this._pending_read_resolver = null;
    this.streamType = streamType;

    
    // if(streamType === 0)
    //   console.log('audio ready for tracks')

    //不管是videotrack还是audiotrack都ready了
    // console.log(this.fileUri+'begin tracks ready')
    await this._tracksReady();
    // console.log(this.fileUri+'finish tracks ready')

    // if(streamType === 0)
    //   console.log('audio finished tracks')

    if (this.streamType == AUDIO_STREAM_TYPE) {
      this._selectTrack(this.audioTrack!);
    } else {
      this._selectTrack(this.videoTrack!);
    }
    // console.log('demuxer initialize finished')
  }

  //这个方法将会返回framerate和bitrate
  getOtherVideoConfig() {
    return {
      bitrate: Math.floor(this.videoTrack!.bitrate),
      framerate: (this.videoTrack!.nb_samples/this.videoTrack!.duration*this.videoTrack!.timescale).toFixed(2)
    }
  }

  override getDecoderConfig(): VideoDecoderConfig|AudioEncoderConfig {

    console.log('output the data...');
    console.log(this.videoTrack!.nb_samples);
    console.log(this.videoTrack!.duration);
    console.log(this.videoTrack!.timescale);
    //判断当前流类型
    if (this.streamType == AUDIO_STREAM_TYPE) {
      // console.log('in audio config ')
      // console.log(this.audioTrack?.codec)
      // console.log(this.audioTrack?.audio.sample_rate)
      // console.log(this.audioTrack?.audio.channel_count)
      // console.log(this.source?.getAudioSpecificConfig())
      return {
        codec: this.audioTrack!.codec,
        sampleRate: this.audioTrack?.audio.sample_rate,
        numberOfChannels: this.audioTrack?.audio.channel_count,
        description: this.source?.getAudioSpecificConfig()
      };
    } else {
      let description: Uint8Array|undefined;
      if (this.videoTrack!.codec.includes('avc1') || this.videoTrack!.codec.includes('avc3'))
        description = this._getAvcDescription(this.source?.getAvccBox());
      else if (this.videoTrack!.codec.includes('hev1') || this.videoTrack!.codec.includes('hvc1'))
        description = this._getHvcDescription(this.source?.getHvccBox());
      else
        description = undefined;
      console.log('current decoderconfig')
      console.log({
        codec: this.videoTrack!.codec,
        codedWidth: this.videoTrack?.track_width,
        codedHeight: this.videoTrack?.track_height,

        // displayWidth: this.videoTrack?.track_width,
        // displayHeight: this.videoTrack?.track_height,
        description: description
      });
        return {
          codec: this.videoTrack!.codec,
          codedWidth: this.videoTrack?.track_width,
          codedHeight: this.videoTrack?.track_height,

          // displayWidth: this.videoTrack?.track_width,
          // displayHeight: this.videoTrack?.track_height,
          description: description
        }
      
    }
  }

  override async getNextChunk() {
    //第一步：直接请求getNextChunk
    // console.log(this.over)
    //这里先注释，搞清楚为什么第一帧不见了

      let sample = await this._readSample();
      if(sample !== null){
        const type = sample?.is_sync ? "key" : "delta";
        const pts_us = (sample?.cts! * 1000000) / sample?.timescale!;
        const duration_us = (sample?.duration! * 1000000) / sample?.timescale!;
        const ChunkType = this.streamType == AUDIO_STREAM_TYPE ? EncodedAudioChunk : EncodedVideoChunk;
        return new ChunkType({
          type: type,
          timestamp: pts_us,
          duration: duration_us,
          data: sample!.data
        });
    }else
      return this.rest_number;
  }

  //这里先定义avccBox是any
  //为什么这里能够work呢？？？
  _getAvcDescription(avccBox: any) {
    console.log('avccbox below')
    console.log(avccBox);
    const stream = new DataStream(undefined, 0, DataStream.BIG_ENDIAN);
    avccBox.write(stream);
    return new Uint8Array(stream.buffer, 8);  // Remove the box header.
  }

  //这个写法同样能够work，不需要使用之前的手动写入具体config的方法
  _getHvcDescription(hvccBox: any) {
    console.log('hvccBox below')
    console.log(hvccBox);
    const stream = new DataStream(undefined, 0, DataStream.BIG_ENDIAN);
    hvccBox.write(stream);
    return new Uint8Array(stream.buffer, 8);  // Remove the box header.
  }


  // _getHvcDescription(hvccBox: any) {
  //   var i, j;
  //   var size = 23;
  //   for (i = 0; i < hvccBox.nalu_arrays.length; i++) {
  //     // nalu length is encoded as a uint16.
  //     size += 3;
  //     for (j = 0; j < hvccBox.nalu_arrays[i].length; j++){
  //       // console.log(hvccBox.nalu_arrays[i]["0"].data.length)
  //       size += (2+hvccBox.nalu_arrays[i][j].data.length)
  //     }
  //   }
  //   console.log(size)

  //   var writer = new Writer(size);

  //   writer.writeUint8(hvccBox.configurationVersion);
  //   console.log(((hvccBox.general_profile_space)<<6)+((hvccBox.general_tier_flag)<<5)+(hvccBox.general_profile_idc))
  //   writer.writeUint8(((hvccBox.general_profile_space)<<6)+((hvccBox.general_tier_flag)<<5)+(hvccBox.general_profile_idc));
    
  //   writer.writeUint32(hvccBox.general_profile_compatibility);
  //   writer.writeUint8Array(hvccBox.general_constraint_indicator);
  //   writer.writeUint8(hvccBox.general_level_idc);
    
  //   //?别人写的是24
  //   writer.writeUint16((15<<12)+(hvccBox.min_spatial_segmentation_idc)); //???
  //   console.log((63<<2)+(hvccBox.parallelismType))
  //   writer.writeUint8((63<<2)+(hvccBox.parallelismType));
  //   writer.writeUint8((63<<2)+(hvccBox.chroma_format_idc));
  //   writer.writeUint8((31<<3)+(hvccBox.bit_depth_luma_minus8));
  //   writer.writeUint8((31<<3)+(hvccBox.bit_depth_chroma_minus8));
  //   writer.writeUint16(hvccBox.avgFrameRate);
  //   writer.writeUint8(((hvccBox.constantFrameRate)<<6)+(((hvccBox.numTemporalLayers))<<3)+((hvccBox.temporalIdNested)<<2)+(hvccBox.lengthSizeMinusOne))
  //   writer.writeUint8(hvccBox.nalu_arrays.length)
  //   for(i = 0; i < hvccBox.nalu_arrays.length; i++){
  //     let current = hvccBox.nalu_arrays[i]
  //     console.log(((current.completeness)<<7)+(current.nalu_type))
  //     writer.writeUint8(((current.completeness)<<7)+(current.nalu_type))

  //     writer.writeUint16(current.length)
  //     for(j = 0; j < current.length; j++){
  //       console.log(111111)
  //       console.log((current[j].data.length))
  //       writer.writeUint16(current[j].data.length)
  //       writer.writeUint8Array(current[j].data)
  //       console.log(22222)
  //     }
  //   }
  //   return writer.getData();
  // }

  async _tracksReady() {
    console.log(this.source)
    let info = await this.source!.getInfo();
    console.log(this.source+'finish get info')
    this.videoTrack = info?.videoTracks[0];
    this.audioTrack = info?.audioTracks[0];
  }

  _selectTrack(track: MP4AudioTrack|MP4VideoTrack) {
    console.assert(!this.selectedTrack, "changing tracks is not implemented");
    this.selectedTrack = track;
    this.source?.selectTrack(track);
  }

  async _readSample(): Promise<MP4Sample|undefined>{
    //第二步：从_readSample获取
    console.assert(this.selectedTrack);
    console.assert(!this._pending_read_resolver);

    //如果readySample.length不为0，就返回
    if (this.readySamples?.length) {
      return Promise.resolve(this.readySamples?.shift()!);
    }
    //如果readySample.length为0，就再去上一层寻找
    console.log('_pending_read_resolver....')
    console.log('this.over')
    console.log(this.over);

    //这里直接给了any
    let promise: Promise<any> = new Promise((resolver) => { this._pending_read_resolver = resolver; });
    // console.log('this._pending_read_resolver');
    // console.log(this._pending_read_resolver);
    
    console.assert(this._pending_read_resolver);

    //bind() 方法创建一个新的函数，在 bind() 被调用时，这个新函数的 this 被指定为 bind() 的第一个参数，而其余参数将作为新函数的参数，供调用时使用。
    if(!this.over){
      this.source?.start(this._onSamples.bind(this));
    }else{
      this._pending_read_resolver!(null);
      this._pending_read_resolver = null;
    }
    return promise;
  }

  _onSamples(samples: MP4Sample[]) {
    
    // debugger;
    const SAMPLE_BUFFER_TARGET_SIZE = 50;

    if(samples.length < 1000) {
      this.rest_number = samples.length
      this.over = true;
      console.log('已经取完全部samples了')
    }
      console.log('samples长度大于0')
      this.readySamples?.push(...samples);
      if (<number>this.readySamples?.length >= SAMPLE_BUFFER_TARGET_SIZE)
        this.source?.stop();

      let firstSampleTime = samples[0].cts * 1000000 / samples[0].timescale ;
      console.log(`adding new ${samples.length} samples (first = ${firstSampleTime}). total = ${this.readySamples?.length}`);

      if (this._pending_read_resolver) {
        const current = this.readySamples?.shift();
        // console.log(current)
        this._pending_read_resolver(current);
        this._pending_read_resolver = null;
      }
    }
  
}

class MP4Source {
  file: MP4File;
  info: MP4Info | null;
  //这里的_info_resolver直接给了any类型，后面根据需要调整
  _info_resolver: any;
  _onSamples: any;

  constructor(buffer: ArrayBuffer) {
    // console.log('in MP4Source')

    this.file = createFile();
    console.log('finish create file')
    // console.log('uri')
    // console.log(uri)
    this.file.onError = console.error.bind(console);
    this.file.onReady = this.onReady.bind(this);
    this.file.onSamples = this.onSamples.bind(this);


    console.log('fetching file');


    const blob = new Blob([buffer]);
    const reader = blob.stream().getReader();
    let offset = 0;
    let mp4File = this.file;

    function appendBuffers({done, value}: ReadableStreamReadResult<Uint8Array>) :Promise<any> | undefined {
      if(done) {
        mp4File.flush();
        return;
      }

      let buf = value.buffer as MP4ArrayBuffer;
      buf.fileStart = offset;

      offset += buf.byteLength;

      mp4File.appendBuffer(buf);

      return reader.read().then(appendBuffers);
    }

    reader.read().then(appendBuffers);
    //以下是读取本地文件的写法
    // fetch(uri).then(response => {
    //   // console.log('fetch responded'+uri);
    //   const reader = response.body?.getReader();
    //   let offset = 0;
    //   let mp4File = this.file;

    //   function appendBuffers({done, value}: ReadableStreamReadResult<Uint8Array>) :Promise<any> | undefined{
    //     if(done) {
    //       mp4File.flush();
    //       return;
    //     }

    //     let buf = value.buffer as MP4ArrayBuffer;
    //     buf.fileStart = offset;

    //     offset += buf.byteLength;

    //     mp4File.appendBuffer(buf);

    //     return reader?.read().then(appendBuffers);
    //   }

    //   return reader?.read().then(appendBuffers);
    // })

    console.log('mp4file is ...')
    console.log(mp4File)

    this.info = null;
    this._info_resolver = null;
  }

  onReady(info: MP4Info) {
    // TODO: Generate configuration changes.
    this.info = info;

    if (this._info_resolver) {
      this._info_resolver(info);
      this._info_resolver = null;
    }
  }

  //不确定这里是AudioTrack、videoTrack还是MP4MediaTrack
  selectTrack(track: MP4AudioTrack|MP4VideoTrack) {
    debugLog('selecting track'+track.id);
    this.file.setExtractionOptions(track.id);
  }

  getInfo(): MP4Info | Promise<MP4Info> {
    if (this.info)
      return Promise.resolve(this.info);

    return new Promise((resolver) => { this._info_resolver = resolver; });
  }

  getHvccBox() {
    // TODO: make sure this is coming from the right track.
    console.log(this.file.moov.traks[0].mdia.minf.stbl.stsd.entries[0].hvcC)
    return this.file.moov.traks[0].mdia.minf.stbl.stsd.entries[0].hvcC
  }

  getAvccBox() {
    // TODO: make sure this is coming from the right track.
    return this.file.moov.traks[0].mdia.minf.stbl.stsd.entries[0].avcC
  }

  getAudioSpecificConfig() {
    // TODO: make sure this is coming from the right track.

    // 0x04 is the DecoderConfigDescrTag. Assuming MP4Box always puts this at position 0.
    console.assert(this.file.moov.traks[1].mdia.minf.stbl.stsd.entries[0].esds.esd.descs[0].tag == 0x04);
    // 0x40 is the Audio OTI, per table 5 of ISO 14496-1
    console.assert(this.file.moov.traks[1].mdia.minf.stbl.stsd.entries[0].esds.esd.descs[0].oti == 0x40);
    // 0x05 is the DecSpecificInfoTag
    console.assert(this.file.moov.traks[1].mdia.minf.stbl.stsd.entries[0].esds.esd.descs[0].descs[0].tag == 0x05);

    return this.file.moov.traks[1].mdia.minf.stbl.stsd.entries[0].esds.esd.descs[0].descs[0].data;
  }

 //source.start
 //    this.source.start(this._onSamples.bind(this));
 //表示可以开始样本处理（分段或提取）。 已经收到的样本数据将被处理，新的缓冲区追加操作也将触发样本处理。
  start(onSamples: (samples: MP4Sample[])=>void) {
    console.log("mp4file started")
    // debugger;
    //_onSamples ： this._onSamples
    this._onSamples = onSamples;
    // this.file.setExtractionOptions(track.id);
    this.file.start();
  }

  stop() {
    this.file.stop();
  }

  //onsamples重写了，之前在这里构建encodedVideoChunk，这里调用_onSamples
  //@ts-ignore
  onSamples(track_id: number, ref: any, samples: MP4Sample[]) {
    // debugger;
    // for (const sample of samples) {
    //   const type = sample.is_sync ? "key" : "delta";

    //   const chunk = new EncodedVideoChunk({
    //     type: type,
    //     timestamp: sample.cts,
    //     duration: sample.duration,
    //     data: sample.data
    //   });

    //   this._onChunk(chunk);
    // }
    this._onSamples(samples)
  }
}

function debugLog(msg: string) {
  if (!ENABLE_DEBUG_LOGGING) {
    return;
  }
  console.debug(msg);
}

//目前没用，但是后面可能会有用，因此，这里先ts-ignore
class Writer {
  data: Uint8Array;
  idx: number;
  size: number;
  constructor(size: number) {
    this.data = new Uint8Array(size);
    this.idx = 0;
    this.size = size;
  }

  getData() {
    if(this.idx != this.size)
      throw "Mismatch between size reserved and sized used"

    return this.data.slice(0, this.idx);
  }

  writeUint8(value: number) {
    this.data.set([value], this.idx);
    this.idx++;
  }

  writeUint16(value: number) {
    // TODO: find a more elegant solution to endianess.
    var arr = new Uint16Array(1);
    arr[0] = value;
    var buffer = new Uint8Array(arr.buffer);
    this.data.set([buffer[1], buffer[0]], this.idx);
    this.idx +=2;
  }

  writeUint32(value: number) {
    var arr = new Uint32Array(1);
    arr[0] = value;
    var buffer = new Uint8Array(arr.buffer);
    this.data.set([buffer[3], buffer[2], buffer[1], buffer[0]], this.idx);
    this.idx +=4;
  }

  writeUint8Array(value: Uint8Array) {
    this.data.set(value, this.idx);
    this.idx += value.length;
  }
}


