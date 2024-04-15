import asyncio
import subprocess
import traceback
import multiprocessing
from concurrent.futures import ProcessPoolExecutor
import grpc
from enums.videotask import VideoTask
from type.video import Video
from type.task import Task
import transcoding_pb2
import transcoding_pb2_grpc
from enums.resolution import Resolution
from enums.video_codec import VideoCodec
from enums.audio_codec import AudioCodec
from enums.bitrate import Bitrate
from enums.mode import Mode
import os

from loguru import logger
from transcode import execute_ez_vod_transcode

# import ipfshttpclient

# 临时队列
queue_list = []

# Coroutines to be invoked when the event loop is shutting down.
_cleanup_coroutines = []

# 创建进程池
executor = ProcessPoolExecutor(max_workers=multiprocessing.cpu_count())


##### This was used in a previous version and is temporarily commented out.
##### If it's not applicable to the current version, it will be removed.

# def build_videotask(request: transcoding_pb2.DispatchVoDRequest):
#     # 这里需要将request转换成videotask
#     resolution = ""
#     if request.videoinfo.originresolution == 0:
#         resolution = Resolution.SD
#     elif request.videoinfo.originresolution == 1:
#         resolution = Resolution.HD
#     elif request.videoinfo.originresolution == 2:
#         resolution = Resolution.FHD
#     else:
#         raise ValueError(
#             f"Unsupported resolution: {request.videoinfo.originresolution}"
#         )

#     videocodec = ""
#     if request.videoinfo.origincodec == 0:
#         videocodec = VideoCodec.H264
#     elif request.videoinfo.origincodec == 1:
#         videocodec = VideoCodec.H265
#     else:
#         raise ValueError(f"Unsupported codec: {request.videoinfo.origincodec}")

#     audiocodec = ""
#     if request.videoinfo.originaudiocodec == 0:
#         audiocodec = AudioCodec.AAC
#     elif request.videoinfo.originaudiocodec == 1:
#         audiocodec = AudioCodec.NONE
#     else:
#         raise ValueError(
#             f"Unsupported audiocodec: {request.videoinfo.originaudiocodec}"
#         )

#     outputresolution = ""
#     if request.videoinfo.originresolution == 0:
#         outputresolution = Resolution.SD
#     elif request.videoinfo.originresolution == 1:
#         outputresolution = Resolution.HD
#     elif request.videoinfo.originresolution == 2:
#         outputresolution = Resolution.FHD
#     else:
#         raise ValueError(f"Unsupported resolution: {request.outputresolution}")

#     outputcodec = ""
#     if request.outputcodec == 0:
#         outputcodec = VideoCodec.H264
#     elif request.outputcodec == 1:
#         outputcodec = VideoCodec.H265
#     else:
#         raise ValueError(f"Unsupported codec: {request.outputcodec}")

#     bitrate = ""
#     if request.bitrate == 0:
#         bitrate = Bitrate.LOW
#     elif request.bitrate == 1:
#         bitrate = Bitrate.MEDIUM
#     elif request.bitrate == 2:
#         bitrate = Bitrate.HIGH
#     elif request.bitrate == 3:
#         bitrate = Bitrate.ULTRA
#     else:
#         raise ValueError(f"Unsupported bitrate: {request.bitrate}")

#     mode = ""
#     if request.tasktype == 0:
#         mode = Mode.Normal
#     elif request.tasktype == 1:
#         mode = Mode.Latency
#     elif request.tasktype == 2:
#         mode = Mode.Live
#     else:
#         raise ValueError(f"Unsupported mode: {request.tasktype}")

#     video = Video(
#         request.originurl,
#         request.outputurl,
#         resolution,
#         videocodec,
#         request.videoinfo.originbitrate,
#         request.videoinfo.originframerate,
#         request.videoinfo.duration,
#         audiocodec,
#     )
#     task = Task(outputresolution, outputcodec, bitrate, mode)
#     return VideoTask(video, task, request.taskid)


def worker(request: transcoding_pb2.DispatchVoDRequest, inputfile: str):
    print(f"Worker is running in process {os.getpid()}")
    # 具体的转码处理逻辑
    # 目前可以不需要构建videotask这一步骤
    # videotask = build_videotask(request)
    # build outputcodec

    # 首先返回当前队列信息，例如前面有几个任务，预计什么时候能够返回
    # transcoding_pb2.DispatchVoDReply(taskid=request.taskid)

    outputcodec = ""
    if request.outputcodec == 0:
        outputcodec = VideoCodec.H264
    elif request.outputcodec == 1:
        outputcodec = VideoCodec.H265
    else:
        raise ValueError(f"Unsupported codec: {request.outputcodec}")

    # logger.info(f"successfully build {request.taskid} videotask.")
    # 读取本地设备号
    with open("rpc/device.txt", "r") as f:
        mac = f.readline().strip()
    f.close()
    execute_ez_vod_transcode(
        request.taskid, inputfile, outputcodec, mac, request.uniqueid
    )


class Handler(transcoding_pb2_grpc.TranscoderServicer):
    async def DispatchVoDTask(
        self,
        request: transcoding_pb2.DispatchVoDRequest,
        context: grpc.aio.ServicerContext,
    ) -> transcoding_pb2.TaskResult:
        transcoder = Transcoder(request, context)
        logger.info(f"Received DispatchVodTask {request.taskid}")
        logger.info(f"{request}")
        # dispatch_vod_task add task to task_queue
        dispatch_result = transcoder.dispatch_vod_task()
        yield dispatch_result

        print("start to execute vod task")
        transcoder.download_video_via_ipfs()
        execution_result = await transcoder.execute_vod_task()
        yield execution_result
        print("Finished execute vod task")


class Transcoder:
    def __init__(self, request, context):
        self.request = request
        self.context = context
        self.filename = ""

    def download_video_via_ipfs(self):

        current_file_path = os.path.abspath(__file__)
        current_directory = os.path.dirname(current_file_path)
        inputfile = os.path.join(
            current_directory, "source", self.request.originurl + ".mp4"
        )
        # connect to ipfs daemon API server
        # api = ipfshttpclient.connect('/ip4/127.0.0.1/tcp/5001')

        response = subprocess.run(
            ["ipfs", "get", "-o", inputfile, self.request.originurl],
            stdout=subprocess.PIPE,
        ).stdout.decode("utf-8")

        print(response)

        # rename the file
        # rename_file_path = os.path.join(os.path.dirname(file_path), self.request.originurl+'.mp4')

        # os.rename(file_path, rename_file_path)
        self.filename = inputfile

    def dispatch_vod_task(self) -> transcoding_pb2.TaskResult:

        # 这里将具体的任务请求加入到队列中
        queue_list.append(self.request)
        logger.info(f"Add {self.request.taskid} to queue.")
        return transcoding_pb2.TaskResult(taskid=self.request.taskid, status=0)

    async def execute_vod_task(self) -> transcoding_pb2.TaskResult:

        # 创建新的进程处理任务
        # future = executor.submit(worker, self.request)
        loop = asyncio.get_event_loop()
        asyncio_future = loop.run_in_executor(
            executor, worker, self.request, self.filename
        )
        try:
            result = await asyncio_future
            logger.success(f"Successfully execute {self.request.taskid} task.")
        except Exception as e:
            logger.error(f"Task failed with exception:{e}\n{traceback.format_exc()}")
        return transcoding_pb2.TaskResult(taskid=self.request.taskid, status=1)


# async def FinishTask(self, request, context):
#     logger.info("Finished task.")
#     print(request)
#     return transcoding_pb2.FinishVoDReply()


async def serve() -> None:
    server = grpc.aio.server()
    transcoding_pb2_grpc.add_TranscoderServicer_to_server(Handler(), server)
    listen_addr = "[::]:50051"
    server.add_insecure_port(listen_addr)
    logger.info(f"Listening on {listen_addr}")
    await server.start()

    async def server_graceful_exit() -> None:
        await server.stop(3)

    _cleanup_coroutines.append(server_graceful_exit)
    await server.wait_for_termination()


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    try:
        loop.run_until_complete(serve())
    finally:
        loop.run_until_complete(asyncio.gather(*_cleanup_coroutines))
        loop.close()
