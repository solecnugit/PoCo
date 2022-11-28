## 针对tool的说明
  目前整个tool目录中，ts文件作用如下：
- audiocontext.ts将能够支持线上视频播放。
- demo.ts控制着整个转码流程的运行。
- mp4_demuxer.ts起到解复用的功能，对输入的MP4视频，将其抽离成一个个sample。
- resolution.ts对编码的视频宽度等做出了规约（这一块是webm-muxer.js的相关套件）。
- SampleLock.ts是自己实现的锁，用于计数解码出的帧的数目。
- type.ts对一些公用的常量进行了定义。
- webm_muxer.ts是webm-muxer.js的核心，主要起到复用的功能。