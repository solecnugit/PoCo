"""
This module provides functions for transcoding video files.
It includes functions to execute the transcoding and to generate the necessary commands.
"""

import os
import json
import random
import datetime
import subprocess
from loguru import logger
from measure import QoSAnalyzer
from enums import VideoCodec, Accelerator
from config_parser import parse_config_file, get_config_value
from hw_capabilities import get_nvenc_capability


def execute_ez_vod_transcode(
    taskid: str, inputurl: str, outputcodec: VideoCodec, contractid: str
):
    outputurl = os.path.dirname(inputurl)
    command, outputpath = prepare_ez_transcode(inputurl, outputurl, outputcodec, taskid)
    logger.info(f"generate {taskid} instruction: {command}.")

    def transcoding():
        subprocess.run(command, shell=True, stdout=subprocess.PIPE, check=True)
        logger.success(f"Transcode {taskid} finished.")

    analyzer = QoSAnalyzer(inputurl, outputpath)
    analyzer.measure(transcoding, contractid)


def prepare_ez_transcode(
    inputurl: str, outputurl: str, outputcodec: VideoCodec, taskid: str
):
    accelerator = get_random_accelerator(outputcodec)
    logger.info(f"Selected {accelerator.value}  for {taskid}.")

    # 访问test.ini获取转码参数
    encode_lib = read_encode_ini()
    logger.info(f"encode_lib is {encode_lib}")
    # 获取具体编码库
    codec = get_config_value(encode_lib, outputcodec.value, accelerator.value)

    logger.info(f"Selected {codec}  for {taskid}.")

    # 获取文件名和后缀
    filename, extension = os.path.splitext(os.path.basename(inputurl))

    # 获取当前时间戳
    timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")

    command = ""
    # if videotask.mode == Mode.Normal:
    outputpath = os.path.join(outputurl, f"{filename}_{timestamp}{extension}")
    command = f"ffmpeg -y -i {inputurl} -c:v {codec} -c:a copy {outputpath}"

    return command, outputpath


def get_random_accelerator(videocodec: VideoCodec):
    """
    从capabilities.json中随机获取一个转码能力。

    Returns:
        accelerator (Accelerator): 转码能力.
    """
    logger.info(f"videocodec {videocodec.value}")
    capability = read_capability()
    logger.info(f"get capability: {capability}")
    capability = json.loads(capability)
    accelerators = capability[videocodec.value]
    logger.info(f"get accelerators: {accelerators}")
    config = random.choice(accelerators)
    if config == "software":
        config = Accelerator.software
    elif config == "nvidia":
        config = Accelerator.nvidia
    elif config == "intel":
        config = Accelerator.intel
    print(config)
    return config


def read_capability():
    """
    读取capabilities.json，返回由转码能力组成的对象。

    Returns:
        capability (dict): 转码能力组成的对象.
    """
    parent_path = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    file_path = os.path.join(parent_path, "capabilities.json")
    if os.path.exists(file_path):
        with open(file_path, "r", encoding="utf-8") as f:
            capability = json.load(f)
        f.close()
    else:
        capability = get_nvenc_capability()
    return capability


def read_encode_ini():
    """
    读取encode.ini，返回由转码参数组成的对象。

    Returns:
        encode_lib (dict): 转码参数组成的对象.
    """
    current_path = os.path.dirname(os.path.abspath(__file__))
    file_path = os.path.join(current_path, "test.ini")
    logger.info(f"file_path is {file_path}")
    try:
        encode_lib = parse_config_file(file_path)
    except FileNotFoundError as exc:
        raise FileNotFoundError(f"文件{file_path}不存在") from exc

    return encode_lib


# 这里的代码改自transcode/transcode.py
# def execute_vod_transcode(videotask: VideoTask, mac: str, contractid: str):

# command, outputpath = prepare_transcode(videotask, mac, contractid)
# logger.info(f"finish build {videotask.taskid} instruction: {command}.")
# print(videotask.outputcodec)

# 创建QoSAnalyzer对象
# 但是，实际上的QoSAnalyzer评估，应该由实际server做一份，并且作为证明的一部分返回，但是并不应该在这里进行？
# analyzer = QoSAnalyzer(videotask, outputpath)

# subprocess.run(command, shell=True, stdout=subprocess.PIPE)
# logger.success(f"Transcode {videotask.taskid} finished.")

#  这里把具体的transcode指令作为参数传入
# callback_func = functools.partial(handle_transcode, command)
# if videotask.mode == Mode.Normal:
#     analyzer.measure(callback_func, contractid)
# elif videotask.mode == Mode.Latency:
#     analyzer.measure_latency(callback_func, contractid, outputpath)

# print(qosmetric)
# helper = MySQLHelper()
# helper.connect()
# helper.insert_metric(qosmetric)
# # helper.disconnect()

# 这里需要更改数据库任务结果
# helper = MySQLHelper()
# helper.connect()
# helper.update_mac_task(videotask.taskid, mac)
# helper.disconnect()


# def prepare_transcode(videotask: VideoTask, mac: str, contractid: str):
#     task_outputcodec = videotask.outputcodec
#     task_resolution = videotask.outputresolution
#     task_bitrate = videotask.bitrate
#     accelerator = get_random_accelerator(task_outputcodec)
#     logger.info(f"Selected {accelerator.value}  for {videotask.taskid}.")

#     # 访问test.ini获取转码参数
#     encode_lib = read_encode_ini()
#     logger.info(f"encode_lib is {encode_lib}")
#     # 获取具体编码库
#     codec = get_config_value(encode_lib, task_outputcodec.value, accelerator.value)

#     logger.info(f"Selected {codec}  for {videotask.taskid}.")
#     # 获取具体比特率
#     bitrate = get_config_value(encode_lib, task_resolution.value, task_bitrate.value)

#     path = videotask.path

#     # 获取文件名和后缀
#     filename, extension = os.path.splitext(os.path.basename(path))

#     # 获取当前时间戳
#     timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")

#     command = ""
#     if videotask.mode == Mode.Normal:
#         outputpath = os.path.join(
#             videotask.outputpath, f"{filename}_{timestamp}{extension}"
#         )
#         command = "ffmpeg -y -i {} -c:v {} -b:v {} -c:a copy {}".format(
#             path, codec, bitrate, outputpath
#         )
# 暂时不考虑latency场景，假定所有的都是
# elif videotask.mode == Mode.Latency:
#     outputpath = os.path.join(videotask.outputpath, f"{filename}_{timestamp}")
#     if not os.path.exists(outputpath):
#         os.mkdir(outputpath)
#     build_m3u8(outputpath, float(videotask.duration))
#     command = "ffmpeg -y -i {} -c:v {} -b:v {} -c:a copy -f segment -segment_time 10 -segment_list {}/out.m3u8 -segment_format mpegts {}/output_%03d.ts".format(path, codec, bitrate, outputpath, outputpath)

# print("当前command")
# print(outputpath)
# return command, outputpath
