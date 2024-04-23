from enum import Enum


class AudioCodec(Enum):
    AAC = "aac"
    NONE = "none"

    def __str__(self):
        return self.value
