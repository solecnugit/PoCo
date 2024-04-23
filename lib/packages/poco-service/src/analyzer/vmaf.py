import os
import subprocess
import concurrent.futures
import time
import re
from video_quality_analyzer import VideoQualityAnalyzer
from ..key_frame_extractor import KeyFrameExtractor


class VMAFAnalyzer(VideoQualityAnalyzer):
    def analyze(self, origin_video, transcoded_video):
        # ...
        output_file = self._get_vmaf(origin_video, transcoded_video)
        quality = self.showResult(output_file)
        return quality

    def _get_vmaf(self, origin_video, transcoded_video):
        """
        读取原始视频和转码后视频的路径，执行ffmpeg vmaf命令，获取视频质量分析结果。
        这里默认开启多线程，线程数为8。

        Args:
            origin_video (str): 原始视频的路径.
            transcoded_video (str): 转码后视频的路径.

        Returns:
            output_file (str): 视频质量分析结果路径.

        """
        # ...
        command = f'ffmpeg -nostats -i {transcoded_video} -i {origin_video} -lavfi libvmaf="feature=name=psnr:n_threads=8" -loglevel info -f null - 2> {transcoded_video.split(".")[0]}-vmaf-result.txt'
        print(f"当前执行vmaf指令：{command}")
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE, check=True)
        return transcoded_video.split(".")[0] + "-vmaf-result.txt"

    def _run_vmaf(self, origin_video, transcoded_video, vmaf_path):
        command = f'ffmpeg -i "{transcoded_video}" -i "{origin_video}" -lavfi "[0:v][1:v]libvmaf=psnr=1:n_threads=8:log_path={vmaf_path}" -f null -'
        print("当前执行指令：{command}")
        subprocess.run(command, shell=True, check=True)

    def _calculate_final_vmaf(self, scores, numbers):
        # 计算分数加权平均值
        weighted_sum = 0
        total_weight = sum(numbers)
        for score, weight in zip(scores, numbers):
            weighted_sum += score * weight
        weighted_average = "{:.6f}".format(weighted_sum / total_weight)
        return weighted_average

    def key_frame_vmaf(self, origin_video, transcoded_video):
        """
        读取原始视频和转码后视频的路径，执行ffmpeg vmaf命令，获取视频质量分析结果。
        使用跳帧的方式，只对关键帧区间的帧进行vmaf分析。

        Args:
            origin_video (str): 原始视频的路径.
            transcoded_video (str): 转码后视频的路径.

        Returns:
            output_file (str): 视频质量分析结果路径.

        """
        key_extractor = KeyFrameExtractor(origin_video, transcoded_video)
        out_dir, key_time = key_extractor.extractFrame()
        source_dir = os.path.join(out_dir, "source")
        target_dir = os.path.join(out_dir, "target")
        vmaf_dir = os.path.join(out_dir, "vmaf")

        with concurrent.futures.ProcessPoolExecutor() as executor:
            futures = []
            for slip in sorted(os.listdir(source_dir)):
                futures.append(
                    executor.submit(
                        self._run_vmaf,
                        source_dir + "/" + slip,
                        target_dir + "/" + slip,
                        vmaf_dir + "/" + slip.split(".")[0] + ".txt",
                    )
                )
            # 等待任务完成
            for future in futures:
                future.result()
        executor.shutdown()

        vmaf_slip_list = []
        count_list = []
        start_time = time.time()
        for vmaf_txt in sorted(os.listdir(vmaf_dir)):
            vmaf_txt_path = os.path.join(vmaf_dir, vmaf_txt)
            # 可以用shell写，也可以用python写，分别对比一下花费的时间
            # 这里先使用shell实现
            with open(vmaf_txt_path, "r") as f:
                lines = f.readlines()
                # 从vmaf的txt文件中提取vmaf分数
                vmaf_str = lines[5].split()[14]
                m = re.search(r"\d+(\.\d+)?", vmaf_str)
                vmaf = float(m.group(0))
                vmaf_slip_list.append(vmaf)
            f.close()
        # 获取每个slip的vmaf分数后，只需要获得对应的slip的帧数即可
        count_list = key_extractor.get_frame_number(key_time)
        print(count_list)
        result = self._calculate_final_vmaf(vmaf_slip_list, count_list)
        print("VMAF分析结果：{}".format(result))

    def showResult(self, output_file):
        """
        针对每个视频质量分析结果，提取视频质量数据，并且输出。

        Args:
            output_file (str): 视频质量分析结果路径.

        Returns:
            Result (str): 视频质量分析结果.

        """
        print(output_file)
        result = ""
        with open(output_file, "r") as f:
            lines = f.readlines()
            last_line = lines[-3]
            elements = last_line.split()
            # print(elements)
            result = elements[-1]
            print("VMAF分析结果：{}".format(result))
        f.close()
        return result
