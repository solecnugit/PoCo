from abc import ABC, abstractmethod

class VideoQualityAnalyzer(ABC):
    """
        这段代码定义了一个名为VideoQualityAnalyzer的抽象基类，
        其中包含了抽象方法analyze和showResult。
    """
    @abstractmethod
    # 假定传入的参数是两个视频的路径
    def analyze(self, origin_video, transcoded_video):
        """
            使用具体的视频评估算法分析视频质量。

            Args:
                origin_video (str): 原始视频的路径.
                transcoded_video (str): 转码后视频的路径.

            Returns:
                Result (str): 视频质量分析结果.

        """
        pass

    def showResult(self, output_file):
        """
            针对每个视频质量分析结果，提取视频质量数据，并且输出。

            Args:
                output_file (str): 视频质量分析结果路径.

            Returns:
                Null

        """
        pass