from enum import Enum

class VideoQualityKind(Enum):
    VMAF = "vmaf"
    SSIM = "ssim"
    MSSSIM = "msssim"
