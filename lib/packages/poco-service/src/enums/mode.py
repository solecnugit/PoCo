from enum import Enum

class Mode(Enum):
    Normal = "normal"
    Latency = "latency-critical"
    Live = "live"