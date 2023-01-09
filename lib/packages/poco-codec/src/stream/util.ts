import {createFFmpeg, fetchFile} from "@ffmpeg/ffmpeg"
import { sha256 } from "js-sha256"



// split应该要根据带宽计算出以多少秒进行划分，这可以在另一个函数中实现
// split实现了对文件的切分，切分后的每一个文件应当被分别分发出去。
// 在split的过程中，返回n个切片，每个切片将会作为一个独立的job进行transocde，也因此具有独特的jobid
export async function splitVideo(video: File){
    // 这个name应该是要拿出去的
    const name = video.name;

    // 加载ffmpeg相关组件
    const ffmpeg = createFFmpeg({ 
      corePath: 'https://unpkg.com/@ffmpeg/core@0.11.0/dist/ffmpeg-core.js',
      log: true 
    });
    await ffmpeg.load();

    //将上传到的文件读取到ffmpeg.wasm中
    ffmpeg.FS('writeFile', name, await fetchFile(video));
    await ffmpeg.run('-i', name,  '-acodec', 'copy', '-vcodec', 'copy', 
  '-f', 'segment', '-segment_time', '7', '-reset_timestamps', '1', '-map', '0:0', '-map', '0:1', '-segment_list', 'output.list',  'output-%d.mp4');
        
    //  list 经过切分后 格式如后 ['output-0.mp4', 'output-1.mp4', '']
    //  加入{encoding: ""}报错，表示只有两个参数？
    //@ts-ignore
    const list = ffmpeg.FS("readFile", "output.list",  { encoding: 'utf8' }) as string
    console.log(list);
    var filelist = list.split('\n')
    // outputlist 经过切分后，格式如后 output-0.webm output-1.webm，这个将会用作转码后的文件名
    // 这里的replaceall替换成什么要根据最后的转码结果来定
    // 但是这里先写死
    var outputlist = list.replaceAll(".mp4", ".webm");
    var split_outputlist = outputlist.split('\n');
    console.log(split_outputlist);

    // blobarr用来存储转码后的视频
    var fileArr = new Array();
    for(var i = 0; i < filelist.length - 1; i++){
        //使用readFile读取每个文件
        const buffer = ffmpeg.FS('readFile', filelist[i]);

        //@ts-ignore
        fileArr[i] = new File(buffer, filelist[i]);
    }

    return fileArr;

}

export function getFileId(name: string){
    var current_date = (new Date()).valueOf().toString();
    var random = Math.random().toString();
    const hasher = sha256.create()
    hasher.update(current_date);
    hasher.update(random);
    hasher.update(name);
    return hasher.hex();
}


export function merge(){

}