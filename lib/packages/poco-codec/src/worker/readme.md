## 针对worker的说明
  目前整个tool目录中，ts文件作用如下：
- audio_transcoder控制着audio的转码过程。
- demux_decode_worker创建并且传递audio_transcodervideo_transcoder与主线程的通讯信息。
- encoder-worker控制着编码过程（已经融合进了audio_transcoder以及video_transcoder中了）。
- video_transcoder控制着video的转码过程。
- webm-worker控制着webm的封装过程
- tsconfig.json比较特殊，因为这里都是worker，在worker环境下，将lib从dom改成了webWorker