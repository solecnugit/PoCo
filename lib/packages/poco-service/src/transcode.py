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
from enums import VideoCodec, Accelerator
from config_parser import parse_config_file, get_config_value
from hw_capabilities import get_nvenc_capability


def execute_ez_vod_transcode(
    taskid: str, inputurl: str, outputcodec: VideoCodec, contractid: str
):
    outputurl = os.path.dirname(inputurl)
    command, outputpath = generate_transcode_command(
        inputurl, outputurl, outputcodec, taskid
    )
    logger.info(f"finish build {taskid} instruction: {command}.")

    subprocess.run(command, shell=True, stdout=subprocess.PIPE)
    logger.success(f"Transcode {taskid} finished.")


def generate_transcode_command(
    inputurl: str, outputurl: str, outputcodec: VideoCodec, taskid: str
):
    """
    Generate a command to transcode a video file.

    Args:
        inputurl (str): The URL of the input video file.
        outputurl (str): The URL of the output video file.
        outputcodec (VideoCodec): The codec to use for the output video.
        taskid (str): The ID of the task.

    Returns:
        tuple: The command to execute and the path of the output file.
    """
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
        raise FileNotFoundError(f"file {file_path} doesn't exist.") from exc

    return encode_lib
