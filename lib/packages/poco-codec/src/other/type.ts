import { MP4File, createFile, MP4Sample, MP4Info, MP4VideoTrack, MP4ArrayBuffer } from "mp4box"
// import "dom-webcodecs"

export class MP4Source {
  file: MP4File;
  info: MP4Info | null;
  //这里的_info_resolver直接给了any类型，后面根据需要调整
  _info_resolver: any;
  _onChunk: any;
  constructor(uri: string) {
    this.file = createFile();
    this.file.onError = console.error.bind(console);
    this.file.onReady = this.onReady.bind(this);
    this.file.onSamples = this.onSamples.bind(this);

    fetch(uri).then(response => {
      const reader = response.body?.getReader();
      let offset = 0;
      let mp4File = this.file;

      //这里的value直接给了any类型，后面根据需要调整
      function appendBuffers({done, value}: ReadableStreamReadResult<Uint8Array>): Promise<any> | undefined {
        if (done) {
          mp4File.flush();
          return;
        }

        let buf = value.buffer as MP4ArrayBuffer;
        buf.fileStart = offset;

        offset += buf.byteLength;

        mp4File.appendBuffer(buf);

        return reader?.read().then(appendBuffers);
      }

      return reader?.read().then(appendBuffers);
    })

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

  getInfo(): Promise<MP4Info> {
    if (this.info)
      return Promise.resolve(this.info);

    return new Promise((resolver) => { this._info_resolver = resolver; });
  }

  getAvccBox() {
    // TODO: make sure this is coming from the right track.
    //如果是除了hevc，h264以外的视频，无法处理
    return this.file.moov.traks[0].mdia.minf.stbl.stsd.entries[0].avcC
  }

  getHvccBox() {
    return this.file.moov.traks[0].mdia.minf.stbl.stsd.entries[0].hvcC
  }

  //onChunk这里给了any类型
  start(track: any, onChunk: any) {
    this._onChunk = onChunk;
    this.file.setExtractionOptions(track.id);
    this.file.start();
  }

  //这里必须要用track_id和ref接收，所以使用了ts-ignore
  //@ts-ignore
  onSamples(track_id: number, ref: any, samples: MP4Sample[]) {
    for (const sample of samples) {
      const type = sample.is_sync ? "key" : "delta";

      const chunk = new EncodedVideoChunk({
        type: type,
        timestamp: sample.cts,
        duration: sample.duration,
        data: sample.data
      });

      this._onChunk(chunk);
    }
  }
}

export class MP4Demuxer {
  source: MP4Source;
  track: MP4VideoTrack | undefined;
  constructor(uri: string) {
    this.source = new MP4Source(uri);
  }

  //这里的box没有合适的index.d.ts，直接给了any类型
  getAvccExtradata(avccBox: any) {
    var i;
    var size = 7;
    for (i = 0; i < avccBox.SPS.length; i++) {
      // nalu length is encoded as a uint16.
      size += 2 + avccBox.SPS[i].length;
    }
    for (i = 0; i < avccBox.PPS.length; i++) {
      // nalu length is encoded as a uint16.
      size += 2 + avccBox.PPS[i].length;
    }

    var writer = new Writer(size);

    writer.writeUint8(avccBox.configurationVersion);
    writer.writeUint8(avccBox.AVCProfileIndication);
    writer.writeUint8(avccBox.profile_compatibility);
    writer.writeUint8(avccBox.AVCLevelIndication);
    writer.writeUint8(avccBox.lengthSizeMinusOne + (63 << 2));

    writer.writeUint8(avccBox.nb_SPS_nalus + (7 << 5));
    for (i = 0; i < avccBox.SPS.length; i++) {
      writer.writeUint16(avccBox.SPS[i].length);
      writer.writeUint8Array(avccBox.SPS[i].nalu);
    }

    writer.writeUint8(avccBox.nb_PPS_nalus);
    for (i = 0; i < avccBox.PPS.length; i++) {
      writer.writeUint16(avccBox.PPS[i].length);
      writer.writeUint8Array(avccBox.PPS[i].nalu);
    }

    return writer.getData();
  }

  getHvccExtradata(hvccBox: any) {
    var i, j;
    var size = 23;
    for (i = 0; i < hvccBox.nalu_arrays.length; i++) {
      // nalu length is encoded as a uint16.
      size += 3;
      for (j = 0; j < hvccBox.nalu_arrays[i].length; j++){
        // console.log(hvccBox.nalu_arrays[i]["0"].data.length)
        size += (2+hvccBox.nalu_arrays[i][j].data.length)
      }
    }
    console.log(size)

    var writer = new Writer(size);

    writer.writeUint8(hvccBox.configurationVersion);
    console.log(((hvccBox.general_profile_space)<<6)+((hvccBox.general_tier_flag)<<5)+(hvccBox.general_profile_idc))
    writer.writeUint8(((hvccBox.general_profile_space)<<6)+((hvccBox.general_tier_flag)<<5)+(hvccBox.general_profile_idc));
    
    writer.writeUint32(hvccBox.general_profile_compatibility);
    writer.writeUint8Array(hvccBox.general_constraint_indicator);
    writer.writeUint8(hvccBox.general_level_idc);
    
    //?别人写的是24
    writer.writeUint16((15<<12)+(hvccBox.min_spatial_segmentation_idc)); //???
    console.log((63<<2)+(hvccBox.parallelismType))
    writer.writeUint8((63<<2)+(hvccBox.parallelismType));
    writer.writeUint8((63<<2)+(hvccBox.chroma_format_idc));
    writer.writeUint8((31<<3)+(hvccBox.bit_depth_luma_minus8));
    writer.writeUint8((31<<3)+(hvccBox.bit_depth_chroma_minus8));
    writer.writeUint16(hvccBox.avgFrameRate);
    writer.writeUint8(((hvccBox.constantFrameRate)<<6)+(((hvccBox.numTemporalLayers))<<3)+((hvccBox.temporalIdNested)<<2)+(hvccBox.lengthSizeMinusOne))
    writer.writeUint8(hvccBox.nalu_arrays.length)
    for(i = 0; i < hvccBox.nalu_arrays.length; i++){
      let current = hvccBox.nalu_arrays[i]
      console.log(((current.completeness)<<7)+(current.nalu_type))
      writer.writeUint8(((current.completeness)<<7)+(current.nalu_type))

      writer.writeUint16(current.length)
      for(j = 0; j < current.length; j++){
        console.log(111111)
        console.log((current[j].data.length))
        writer.writeUint16(current[j].data.length)
        writer.writeUint8Array(current[j].data)
        console.log(22222)
      }
    }
    return writer.getData();
  }

  async getConfig() {
    let info = await this.source.getInfo();
    this.track = info.videoTracks[0];
    if(this.track.codec.includes('hvc1'))
      var extradata = this.getHvccExtradata(this.source.getHvccBox());
    else
      var extradata = this.getAvccExtradata(this.source.getAvccBox());

    let config = {
      codec: this.track.codec,
      codedHeight: this.track.video.height,
      codedWidth: this.track.video.width,
      description: extradata,
    }

    return Promise.resolve(config);
  }

  //这里给了any类型
  start(onChunk: any) {
    this.source.start(this.track, onChunk);
  }
}

export class Writer {
  data: Uint8Array;
  idx: number;
  size: number;
  constructor(size: number) {
    this.data = new Uint8Array(size);
    this.idx = 0;
    this.size = size;
  }

  getData() {
    if (this.idx != this.size)
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
    this.idx += 2;
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