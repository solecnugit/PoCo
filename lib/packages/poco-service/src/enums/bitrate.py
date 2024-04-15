from enum import Enum

# 这里的bitrate是对转码限定的约束，因为源视频的分辨率很难用具体的几种进行规约
class Bitrate(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    ULTRA = "ultra"