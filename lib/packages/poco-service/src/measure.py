import subprocess
from contextlib import contextmanager
import time
import json
from qosmetric import QualityOfServiceMetric
from enums import Mode, AudioCodec

# from .config import get_config_value, parse_config_file
from enums import VideoQualityKind, AudioQualityKind
from analyzer import (
    VMAFAnalyzer,
    NoneAnalyzer,
    SSIMAnalyzer,
    PSNRAnalyzer,
    PESQAnalyzer,
)

# import os

# from .videotask import VideoTask

# from db.mysqlhelper import MySQLHelper


class QoSAnalyzer:
    """
    Analyzes the Quality of Service (QoS) for video processing tasks.

    Attributes
    ----------
    origin_video_path : str
        The path to the original video file.

    output_video_path : str
        The path to the output video file.
    """

    def __init__(self, inputpath: str, outputpath: str):
        self.origin_video_path = inputpath
        self.output_video_path = outputpath
        # self.vquality_map = {
        #     VideoQualityKind.VMAF: VMAFAnalyzer,
        #     VideoQualityKind.SSIM: SSIMAnalyzer,
        # }
        # self.audiocodec = videotask.audiocodec
        # self.aquality_map = {AudioQualityKind.PESQ: PESQAnalyzer}
        # self.get_analyzer_config()

        self.video_quality_kind = VideoQualityKind.VMAF

        # self.audio_quality_kind = AudioQualityKind.PESQ
        # assume input video has no audio
        self.audio_quality_kind = AudioCodec.NONE
        self.video_analyzer = VMAFAnalyzer()
        self.audio_analyzer = NoneAnalyzer()
        # self.audio_analyzer = PESQAnalyzer()

    def get_media_info(self, video_path: str):
        # 获取帧率
        frame_rate_command = ["mediainfo", "--Inform=Video;%FrameRate%", video_path]
        frame_rate = subprocess.run(
            frame_rate_command, capture_output=True, text=True, check=True
        ).stdout.strip()

        # 获取持续时间
        duration_command = ["mediainfo", "--Output=General;%Duration%", video_path]
        duration = subprocess.run(
            duration_command, capture_output=True, text=True, check=True
        ).stdout.strip()

        # 获取帧数  mediainfo --Output="Video;%FrameCount%"
        frame_count_command = ["mediainfo", "--Output=Video;%FrameCount%", video_path]
        frame_count = subprocess.run(
            frame_count_command, capture_output=True, text=True, check=True
        ).stdout.strip()

        return frame_rate, duration, frame_count

    @contextmanager
    def measure(self, cb, contractid: str):
        start_time = time.monotonic()
        # callback function to perform transcoding
        cb()
        end_time = time.monotonic()
        # measure time
        elapsed_time = end_time - start_time

        # using mediainfo to extract bitrate & duration & frameNumber
        (origin_framerate, origin_duration, origin_frmaecount) = self.get_media_info(
            self.origin_video_path
        )
        (output_framerate, output_duration, output_frmaecount) = self.get_media_info(
            self.output_video_path
        )

        # print(elapsed_time)

        # measure video quality and audio quality
        videoquality = self.measure_video_quality()
        audioquality = self.measure_audio_quality()
        # if self.audio_quality_kind != AudioCodec.NONE:
        #     audioquality = self.measure_audio_quality()
        # else:
        #     audioquality = None
        audioquality = None

        qosmetric = QualityOfServiceMetric(
            # contractId is transcoding task id in contract
            contractid,
            elapsed_time,
            self.video_quality_kind,
            self.audio_quality_kind,
            videoquality,
            audioquality,
            origin_framerate,
            origin_duration,
            origin_frmaecount,
            output_framerate,
            output_duration,
            output_frmaecount,
        )
        qosmetric_dict = qosmetric.to_dict()

        # Convert the dictionary to a JSON string
        qosmetric_json = json.dumps(qosmetric_dict, default=str, indent=4)

        # Print the JSON string
        print(qosmetric_json)

    def measure_video_quality(self):
        return self.video_analyzer.analyze(
            self.origin_video_path, self.output_video_path
        )

    def measure_audio_quality(self):
        return self.audio_analyzer.analyze(
            self.origin_video_path, self.output_video_path
        )

    """
        not applicable for current mode
    """

    # def insert_metric_into_db(self, qosmetric: QualityOfServiceMetric):
    #     helper = MySQLHelper()
    #     helper.connect()
    #     helper.insert_metric(qosmetric)
    #     helper.disconnect()

    # def measure_latency(self, cb, contractid: str, outputpath: str):
    #     origin_file_size = os.path.getsize(self.origin_video_path)
    #     start_time = time.monotonic()
    #     t = NewThread(self.wait_first_ts, outputpath)
    #     t.start()
    #     # t.start()
    #     cb()
    #     t.join()
    #     end_time = time.monotonic()
    #     first_ts_time = t.get_result() - start_time
    #     elapsed_time = end_time - start_time
    #     start_time = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    #     output_video_path = self.merge_ts(outputpath)
    #     assert os.path.exists(output_video_path), "合并ts文件失败"
    #     output_file_size = os.path.getsize(output_video_path)
    #     self.output_video_path = os.path.join(outputpath, "output.mp4")
    #     videoquality = self.measure_video_quality()
    #     if self.audiocodec != AudioCodec.NONE:
    #         audioquality = self.measure_audio_quality()
    #     else:
    #         audioquality = None
    #     print("测量 finished")
    #     # 这里的metric可能要改一下
    #     print("start_time{}".format(start_time))
    #     print("first_ts_time{}".format(first_ts_time))
    #     print("elapsed_time{}".format(elapsed_time))
    #     print("origin_file_size{}".format(origin_file_size))
    #     print("output_file_size{}".format(output_file_size))
    #     print("video_analyzer_kind{}".format(self.video_analyzer_kind))
    #     print("videoquality{}".format(videoquality))
    #     print("audio_analyzer_kind{}".format(self.audio_analyzer_kind))
    #     print("audioquality{}".format(audioquality))
    # print(output_file_size)

    """
        not applicable for current mode
    """

    # def merge_ts(self, outputpath: str):
    #     # 读取outputpath目录下的playlist.m3u8，获取ts文件列表
    #     playlist_path = os.path.join(outputpath, "playlist.m3u8")
    #     with open(playlist_path, "r") as f:
    #         lines = f.readlines()
    #     ts_list = []
    #     for line in lines:
    #         if line.startswith("output_") and line.endswith(".ts\n"):
    #             ts_list.append(line.strip())
    #     output_file_path = os.path.join(outputpath, "ts_files.txt")
    #     with open(output_file_path, "w") as f:
    #         for file in ts_list:
    #             f.write(f"file '{file}'\n")

    #     # 使用ffmpeg将ts文件合并为mp4文件
    #     output_video_path = os.path.join(outputpath, "output.mp4")
    #     cmd = f"ffmpeg -f concat -safe 0 -i {output_file_path} -c copy {output_video_path}"
    #     os.system(cmd)
    #     return output_video_path

    """
        not applicable for current mode
    """

    # def get_analyzer_config(self):
    #     """
    #     根据输入的Mode从配置文件中获取视频和音频分析工具类型

    #     Args:
    #         None

    #     Returns:
    #         None

    #     """
    #     current_path = os.path.dirname(os.path.abspath(__file__))
    #     file_path = os.path.join(current_path, "analyzer.ini")
    #     analyzer_lib = parse_config_file(file_path)
    #     # 获取具体视频和音频分析工具
    #     video_analyzer = get_config_value(analyzer_lib, self.mode.value, "video")
    #     audio_analyzer = get_config_value(analyzer_lib, self.mode.value, "audio")

    #     # 根据配置文件中的值，将字符串转换为枚举类型
    #     self.video_analyzer_kind = VideoQualityKind(video_analyzer)
    #     self.audio_analyzer_kind = AudioQualityKind(audio_analyzer)

    #     video_analyzer_class = self.vquality_map[self.video_analyzer_kind]
    #     audio_analyzer_class = self.aquality_map[self.audio_analyzer_kind]

    #     self.video_analyzer = video_analyzer_class()
    #     self.audio_analyzer = audio_analyzer_class()

    #     print(self.video_analyzer)
    #     print(self.audio_analyzer)
    # self.qos = QualityOfService(video_analyzer, audio_analyzer)

    """
        not applicable for current mode
    """

    # def wait_first_ts(
    #     self,
    #     outputpath: str,
    # ):
    #     while not os.path.exists(os.path.join(outputpath, "output_000.ts")):
    #         time.sleep(0.05)
    #     print("output_000.ts已经出现，记录时间")
    #     return time.monotonic()


"""
    this class is designed for measure first ts latency and video/audio quality.
    not applicable for current normal situation.
"""
# class NewThread(threading.Thread):
#     def __init__(self, target, args):
#         threading.Thread.__init__(self)
#         self.target = target
#         self.args = args

#     def run(self):
#         print("run运行时...")
#         print(self.args)
#         self.result = self.target(self.args)

#     def get_result(self):
#         try:
#             return self.result
#         except Exception:
#             return None
