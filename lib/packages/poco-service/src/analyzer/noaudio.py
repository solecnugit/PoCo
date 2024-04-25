"""
 AudioQualityAnalyzer for videos without audio.
"""

from analyzer.audio_quality_analyzer import AudioQualityAnalyzer

# import os
# import subprocess
# import time


class NoneAnalyzer(AudioQualityAnalyzer):
    """
    A subclass of AudioQualityAnalyzer for videos without audio.
    """

    def analyze(self, origin_video, transcoded_video):
        print("The current video does not contain audio.")
        return None

    def _extract_audio(self, origin_video, transcoded_video):
        print("not applicable for current video")
        return origin_video, transcoded_video

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
