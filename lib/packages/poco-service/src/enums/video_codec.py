from enum import Enum


class VideoCodec(Enum):
    H264 = "h264"
    H265 = "h265"

    def __str__(self):
        return self.value
