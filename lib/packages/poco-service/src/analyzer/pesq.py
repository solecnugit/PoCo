"""
subclass of AudioQualityAnalyzer that analyzes the audio quality of videos using the PESQ method.
"""

import subprocess
from scipy.io import wavfile
from pesq import pesq
from audio_quality_analyzer import AudioQualityAnalyzer

# import os

# import time


class PESQAnalyzer(AudioQualityAnalyzer):
    """
    Analyzes the audio quality of videos using the PESQ method.
    """

    def _extract_audio(self, origin_video, transcoded_video):
        """
        读取原始视频和转码后视频的路径，使用ffmpeg将音频提取并且转换成WAV格式。

        Args:
            origin_video (str): 原始视频的路径.
            transcoded_video (str): 转码后视频的路径.

        Returns:
            origin_audio (str): 原始视频提取出的音频的路径
            transcoded_audio(str): 转码后视频提取出的音频的路径

        """
        # 将声道合并为1个声道

        command1 = f"ffmpeg -i {origin_video} -vn -acodec pcm_s16le -ar 16000 -ac 1 {origin_video.split('.')[0]}-origin.wav"
        command2 = f"ffmpeg -i {transcoded_video} -vn -acodec pcm_s16le -ar 16000 -ac 1 {transcoded_video.split('.')[0]}-transcoded.wav"

        print(f"当前extract 执行指令：{command1}")
        print(f"当前extract 执行指令：{command2}")

        subprocess.run(command1, shell=True, stdout=subprocess.PIPE, check=True)
        subprocess.run(command2, shell=True, stdout=subprocess.PIPE, check=True)

        return (
            f"{origin_video.split('.')[0]}-origin.wav",
            f"{transcoded_video.split('.')[0]}-transcoded.wav",
        )

    def analyze(self, origin_video, transcoded_video):
        # ...
        origin_audio, transcoded_audio = self._extract_audio(
            origin_video, transcoded_video
        )
        # output_file = self._get_pesq(origin_audio, transcoded_audio)
        return self._get_pesq(origin_audio, transcoded_audio)
        # self.showResult(output_file)

    def _get_pesq(self, origin_video, transcoded_video):
        """
        读取原始视频和转码后视频的路径，执行ffmpeg pesq命令，获取视频质量分析结果。

        Args:
            origin_video (str): 原始音频的路径.
            transcoded_video (str): 转码后音频的路径.

        Returns:
            result (str): 音频质量分析结果.

        """
        # ...
        rate, ref = wavfile.read(origin_video)
        rate, deg = wavfile.read(transcoded_video)
        # print('ref.shape')
        # print(ref.shape)
        result = pesq(rate, ref, deg, "wb")
        result = round(result, 3)
        print(f"pesq分析结果为{result}")
        return result

        # command = "ffmpeg -i {} -i {} -lavfi \"ssim\" -f null - 2> {}".format(transcoded_video, origin_video, transcoded_video.split(".")[0]+"-ssim-result.txt")
        # print("当前执行ssim指令：{}".format(command))
        # result = subprocess.run(command, shell=True, stdout=subprocess.PIPE)
        # return transcoded_video.split(".")[0]+"-ssim-result.txt"

    # def showResult(self, output_file):
    #     """
    #         针对每个视频质量分析结果，提取视频质量数据，并且输出。

    #         Args:
    #             output_file (str): 视频质量分析结果路径.

    #         Returns:
    #             Null

    #     """
    #     with open(output_file, "r") as f:
    #         lines = f.readlines()
    #         last_line = lines[-1]
    #         elements = last_line.split()
    #         fifth_element = elements[10]
    #         print("SSIM分析结果：{}".format(fifth_element))
    #         f.close()
