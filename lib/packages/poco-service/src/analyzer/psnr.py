from analyzer.video_quality_analyzer import VideoQualityAnalyzer
import os
import subprocess
import time


class PSNRAnalyzer(VideoQualityAnalyzer):
    def analyze(self, origin_video, transcoded_video):
        # ...
        output_file = self._get_psnr(origin_video, transcoded_video)
        return self.showResult(output_file)

    def _get_psnr(self, origin_video, transcoded_video):
        """
        读取原始视频和转码后视频的路径，执行ffmpeg psnr命令，获取视频质量分析结果。

        Args:
            origin_video (str): 原始视频的路径.
            transcoded_video (str): 转码后视频的路径.

        Returns:
            output_file (str): 视频质量分析结果路径.

        """
        # ...
        # command = 'ffmpeg -i {} -i {} -lavfi "psnr=stats_file={}.txt" -f null -'.format(transcoded_video, origin_video, transcoded_video.split(".")[0]+"-psnr")
        command = 'ffmpeg -i {} -i {} -lavfi "psnr" -f null - 2> {}'.format(
            transcoded_video,
            origin_video,
            transcoded_video.split(".")[0] + "-psnr-result.txt",
        )
        print("当前执行psnr指令：{}".format(command))
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE)
        return transcoded_video.split(".")[0] + "-psnr-result.txt"

    def showResult(self, output_file):
        """
        针对每个视频质量分析结果，提取视频质量数据，并且输出。

        Args:
            output_file (str): 视频质量分析结果路径.

        Returns:
            result(str): PSNR分析结果

        """
        result = ""
        with open(output_file, "r") as f:
            lines = f.readlines()
            last_line = lines[-1]
            elements = last_line.split()
            result = elements[7]
            print("PSNR分析结果：{}".format(result))
            f.close()
        return result
        # print("PSNR分析结果：")
