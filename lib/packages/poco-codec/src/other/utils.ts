import {MP4Demuxer} from './type'
import {WebVideoDecoder} from './decoder'
import {Transcoder} from '../tool/demo'
// import {beginEncode} from './encode'

// import  'dom-webcodecs';

let mediaStream: MediaStream;

//video decode（也许这也应该开两个worker）
export async function decode(location: string) {
    //这里直接读取一个具体的地址，获取文件
    //也许优化方案是读取一个流？
    let demuxer = new MP4Demuxer(location);
    const trackGenerator = new MediaStreamTrackGenerator({ kind: 'video' });
    let framewWriter = trackGenerator.writable.getWriter();
    mediaStream = new MediaStream([trackGenerator])
    let framecount: number = 0;
    let deocodeInit: VideoDecoderInit = {
        output : frame => {
            framewWriter.write(frame);
            framecount++;
            console.log(framecount)
        },
        error : e => console.error(e)
    };
    let decoder = WebVideoDecoder.getInstance(deocodeInit)
    

    demuxer.getConfig().then((config) => {        
        decoder.configure(config);
        demuxer.start((chunk: EncodedVideoChunk) => { decoder.decode(chunk); })
    })
} 

// export async function encode(mediaStream: MediaStream){
//     beginEncode(mediaStream)
// }

//这里添加方法transcodestream，但是依然没有完全实现功能
//因为读取的视频应当同样是流式的。
//以下是对buffer改成流的做出的相应更改，这里先注释
// export async function transcodeStream(buffer: ArrayBuffer) {
//     const transcoder = new Transcoder(buffer);
//     const finalStream = await transcoder.start();
//     return finalStream;
// }

export async function transcode(buffer: ArrayBuffer){
    console.log(buffer)
    // console.log(location);
   const transcoder = new Transcoder(buffer);
   const finalBuffer = await transcoder.start();
   return finalBuffer;
    // encode(mediaStream);
}

export function hahahah(){
    console.log('hahahah')
}