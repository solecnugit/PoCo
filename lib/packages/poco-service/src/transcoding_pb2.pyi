from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

from google.protobuf.timestamp_pb2 import Timestamp
AAC: AudioCodec
AV1: VideoCodec
DESCRIPTOR: _descriptor.FileDescriptor
ENQUEUE: TaskStatus
EVALUATED: TaskStatus
EXECUTED: TaskStatus
FAILED: TaskStatus
FHD: Resolution
H264: VideoCodec
H265: VideoCodec
HD: Resolution
HIGH: Bitrate
LATENCY: TaskType
LIVE: TaskType
LOW: Bitrate
MEDIUM: Bitrate
NONE: AudioCodec
SD: Resolution
ULTRA: Bitrate
VOD: TaskType
VP9: VideoCodec

class DispatchVoDRequest(_message.Message):
    __slots__ = ["originurl", "outputcodec", "taskid", "uniqueid"]
    ORIGINURL_FIELD_NUMBER: _ClassVar[int]
    OUTPUTCODEC_FIELD_NUMBER: _ClassVar[int]
    TASKID_FIELD_NUMBER: _ClassVar[int]
    UNIQUEID_FIELD_NUMBER: _ClassVar[int]
    originurl: str
    outputcodec: VideoCodec
    taskid: str
    uniqueid: str
    def __init__(self, taskid: _Optional[str] = ..., originurl: _Optional[str] = ..., outputcodec: _Optional[_Union[VideoCodec, str]] = ..., uniqueid: _Optional[str] = ...) -> None: ...

class TaskResult(_message.Message):
    __slots__ = ["status", "taskid"]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    TASKID_FIELD_NUMBER: _ClassVar[int]
    status: TaskStatus
    taskid: str
    def __init__(self, status: _Optional[_Union[TaskStatus, str]] = ..., taskid: _Optional[str] = ...) -> None: ...

class VideoInfo(_message.Message):
    __slots__ = ["duration", "originaudiocodec", "originbitrate", "origincodec", "originframerate", "originresolution", "vid"]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    ORIGINAUDIOCODEC_FIELD_NUMBER: _ClassVar[int]
    ORIGINBITRATE_FIELD_NUMBER: _ClassVar[int]
    ORIGINCODEC_FIELD_NUMBER: _ClassVar[int]
    ORIGINFRAMERATE_FIELD_NUMBER: _ClassVar[int]
    ORIGINRESOLUTION_FIELD_NUMBER: _ClassVar[int]
    VID_FIELD_NUMBER: _ClassVar[int]
    duration: str
    originaudiocodec: AudioCodec
    originbitrate: str
    origincodec: VideoCodec
    originframerate: str
    originresolution: Resolution
    vid: str
    def __init__(self, vid: _Optional[str] = ..., duration: _Optional[str] = ..., origincodec: _Optional[_Union[VideoCodec, str]] = ..., originresolution: _Optional[_Union[Resolution, str]] = ..., originaudiocodec: _Optional[_Union[AudioCodec, str]] = ..., originframerate: _Optional[str] = ..., originbitrate: _Optional[str] = ...) -> None: ...

class TaskStatus(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class VideoCodec(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class AudioCodec(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class Resolution(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class Bitrate(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []

class TaskType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
