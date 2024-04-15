import subprocess
import json


def get_nvenc_capability():

    # 整个函数需要额外判断，如果已经存在capabilities.json，那么就不需要再次执行这个函数了
    # 通过探测本地是否存在nvenc编码器，来判断是否支持nvenc编码

    command = "ffmpeg -hide_banner -encoders | grep nvenc > temp_capability.txt"

    # 另外一种想法是通过探测本地的gpu，考虑是否支持nvenc编码
    # 感觉这种方法可以在自动化init搭建依赖使用，大部分情况下使用上面的指令会更方便

    subprocess.run(command, shell=True)
    capabilities = []
    with open("temp_capability.txt", "r") as f:
        for line in f:
            elements = line.split()
            if len(elements) >= 2:
                capabilities.append(elements[1])
                print(elements[1])
    f.close()

    # 删除临时文件
    command = "rm temp_capability.txt"
    subprocess.run(command, shell=True)

    # 生成capabilities
    if "h264_nvenc" in capabilities:
        h264 = {"h264": ["software", "nvidia"]}
    else:
        h264 = {"h264": ["software"]}

    if "hevc_nvenc" in capabilities:
        hevc = {"h265": ["software", "nvidia"]}
    else:
        hevc = {"h265": ["software"]}

    # 合并字典
    result = {**h264, **hevc}

    # 将合并后的字典转换为JSON格式
    result = json.dumps(result)

    with open("capabilities.json", "w") as f:
        json.dump(result, f)
    f.close()

    return result
