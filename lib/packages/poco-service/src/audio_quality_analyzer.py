from abc import ABC, abstractmethod

class AudioQualityAnalyzer(ABC):
    """
        这段代码定义了一个名为AudioQualityAnalyzer的抽象基类，
        其中包含了抽象方法analyze和showResult。
    """
    @abstractmethod
    def _extract_audio(self, origin_video, transcoded_video):
        """
            读取视频路径，执行ffmpeg命令，提取视频的音频。

            Args:
                origin_video (str): 原始视频的路径.
                transcoded_video (str): 转码后视频的路径.

            Returns:
                origin_audio (str): 原始视频提取出的音频的路径
                transcoded_audio(str): 转码后视频提取出的音频的路径

        """
        pass
    @abstractmethod
    # 假定传入的参数是两个视频的路径
    def analyze(self, origin_video, transcoded_video):
        """
            使用具体的视频评估算法分析视频质量。

            Args:
                origin_video (str): 原始音频的路径.
                transcoded_video (str): 转码后音频的路径.

            Returns:
                result (str): 音频质量分析结果.

        """
        pass

    # def showResult(self, output_file):
    #     """
    #         针对每个音频质量分析结果，提取音频质量数据，并且输出。

    #         Args:
    #             output_file (str): 音频质量分析结果路径.

    #         Returns:
    #             Null

    #     """
    #     pass