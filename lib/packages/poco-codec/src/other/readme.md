## 针对other的说明
  目前整个other目录，许多ts代码其实并没有在目前的项目中被使用。但是，整个other没有被删除，因为我认为，日后重构代码的过程中可能需要它进行重写。
- coder与decoder的职能在worker/video-transcoder中直接体现。
- type.ts中的代码包含了MP4Source以及Writer，MP4Demuxer，目前都被webm-muxer.js中的相关组件替换。在下一步的实现中，将会引入wasm来实现mux与demux，因此type.ts可能日后没有作用了。
- util.ts中封装了tool中的demo.ts（转码的主要逻辑），目前仍然是返回buffer，但是在下一步中，将会使用stream对buffer进行替换（完成了一小半）