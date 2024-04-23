import subprocess
import os


class KeyFrameExtractor:
    def __init__(self, origin_video, transcoded_video):
        self.origin_video = origin_video
        self.transcoded_video = transcoded_video
        self._rate = None  # float
        self._length = None  # float

    @property
    def rate(self):
        if self._rate is None:
            rate = subprocess.check_output(
                ["mediainfo", "--Inform=Video;%FrameRate%", self.origin_video]
            )
            self._rate = float(rate.strip())
        return self._rate

    @property
    def length(self):
        if self._length is None:
            output = subprocess.check_output(
                ["mediainfo", "--Inform=Video;%Duration%", self.origin_video]
            )
            self._length = float(output.strip()) / 1000
        return self._length

    def extractFrame(self):
        """
        根据原始视频路径，提取原始视频关键帧和转码后视频关键帧。

        Args:
            origin_video (str): 原始视频的路径.
            transcoded_video (str): 转码后视频的路径.

        Returns:
            origin_key_frame (str): 原视频关键帧路径.
            transcoded_key_frame (str): 转码视频关键帧路径.
        """
        key_time = self._get_key_time()
        middle_time = self._get_middle_time(key_time)
        self._cut_gop(middle_time)
        self.get_frame_number(key_time)
        return os.path.dirname(self.origin_video), key_time
        # key_frame = self._get_key_frame(origin_video, key_time)
        # return key_frame

    def _get_middle_time(self, key_time):
        """
        根据原始视频路径，关键帧路径，获得要抽帧的具体时间。

        Args:
            origin_video (str): 原始视频的路径.
            key_time (str): 视频关键帧信息路径.

        Returns:
            middle_time (str): 视频将要抽帧的信息的路径.
        """

        timestamps = []
        with open(key_time, "r") as f:
            lines = f.readline().strip().split()
            timestamps = [float(x) for x in lines]
        f.close()

        duration = [round(j - i, 3) for i, j in zip(timestamps[:-1], timestamps[1:])]
        frameofduration = [round(round(d * self.rate) / 2) for d in duration]
        middle = [
            round(t + (d / self.rate), 3)
            for t, d in zip(timestamps[:-1], frameofduration)
        ]
        middle_str = [f"{x:06.3f}" for x in middle]
        output_str = " ".join(middle_str)

        with open(
            os.path.join(self.origin_video.split(".")[0] + "-middletime.txt"), "w"
        ) as f:
            f.write(output_str)
        f.close()

        return self.origin_video.split(".")[0] + "-middletime.txt"

    def _get_key_time(self):
        """
        根据视频路径，读取视频关键帧信息。

        Returns:
            key_time (str): 视频关键帧信息路径.
        """
        if os.path.exists(self.origin_video.split(".")[0] + "-keytime.txt"):
            os.remove(self.origin_video.split(".")[0] + "-keytime.txt")
        command = "ffprobe -skip_frame nokey -select_streams v -show_frames -print_format csv {} | grep -n 'I' | cut -d ',' -f 6 > {}-temp.txt && tr '\n' ' ' < {}-temp.txt >> {}-keytime.txt && rm {}-temp.txt".format(
            self.origin_video,
            self.origin_video.split(".")[0],
            self.origin_video.split(".")[0],
            self.origin_video.split(".")[0],
            self.origin_video.split(".")[0],
        )
        print("当前执行读取视频key frame指令：{}".format(command))
        subprocess.run(command, shell=True, stdout=subprocess.PIPE)

        # 重写时间戳文件，并且添加视频长度信息到文件末尾
        with open(self.origin_video.split(".")[0] + "-keytime.txt", "r") as f:
            lines = f.readline().strip().split()
            timestamps = [float(x) for x in lines]
        timestamps.append(self.length)
        keytime_str_temp = [f"{x:06.3f}" for x in timestamps]
        keytime_str = " ".join(keytime_str_temp)

        with open(self.origin_video.split(".")[0] + "-keytime.txt", "w") as f:
            f.write(keytime_str)
        f.close()

        return self.origin_video.split(".")[0] + "-keytime.txt"

    def _cut_gop(self, middle_time):
        """
        根据视频关键帧信息，截取3帧作为短视频输出到子文件夹中。

        Returns:
            Null
        """
        print(middle_time)
        dir_path = os.path.dirname(middle_time)
        os.makedirs(os.path.join(dir_path, "target"), exist_ok=True)
        os.makedirs(os.path.join(dir_path, "source"), exist_ok=True)
        os.makedirs(os.path.join(dir_path, "vmaf"), exist_ok=True)

        source_dir = os.path.join(dir_path, "source")
        target_dir = os.path.join(dir_path, "target")
        vmaf_dir = os.path.join(dir_path, "vmaf")

        play_time = round(1 / self.rate * 3, 3)

        data = []
        with open(middle_time, "r") as f:
            lines = f.readlines()
            data = list(map(float, lines[0].split()))
        f.close()

        commands = []
        for idx, begin in enumerate(data):
            command = f"ffmpeg -ss {begin} -i {self.origin_video} -t {play_time} -map 0:0 -c:0 copy -map_metadata 0 -default_mode infer_no_subs -ignore_unknown -f mp4 -y {source_dir}/{idx}.mp4"
            commands.append(command)
            # print("当前执行指令1：{}".format(command))
            # subprocess.run(command, shell=True)
            command = f"ffmpeg -ss {begin} -i {self.transcoded_video} -t {play_time} -map 0:0 -c:0 copy -map_metadata 0 -default_mode infer_no_subs -ignore_unknown -f mp4 -y {target_dir}/{idx}.mp4"
            commands.append(command)
            # print("当前执行指令2：{}".format(command))
            # subprocess.run(command, shell=True)
        processes = []
        for command in commands:
            process = subprocess.Popen(command, shell=True)
            processes.append(process)

        for process in processes:
            process.wait()

    def get_frame_number(self, key_time):

        # 生成时间戳文件名
        timestamps = []
        with open(key_time, "r") as f:
            lines = f.readline().strip().split()
            timestamps = [float(x) for x in lines]
        f.close()
        duration = [round(j - i, 3) for i, j in zip(timestamps[:-1], timestamps[1:])]
        frames = [int(round(d * self.rate)) for d in duration]
        print(frames)
        return frames
