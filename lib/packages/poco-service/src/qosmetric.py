"""
 defines the QualityOfServiceMetric class which represents the quality of service metrics 
 for a video processing task.
"""

from typing import Optional
from enums import VideoQualityKind
from enums import AudioQualityKind


class QualityOfServiceMetric:
    """
    Represents the Quality of Service metrics for a video processing task.
    """

    def __init__(
        self,
        contract_id: str,
        execution_time: float,
        video_quality_kind: VideoQualityKind,
        audio_quality_kind: AudioQualityKind,
        video_quality: float,
        audio_quality: Optional[float],
        origin_frame_rate: float,
        origin_duration: float,
        origin_frame_count: int,
        output_frame_rate: float,
        output_duration: float,
        output_frame_count: int,
    ):
        self._contract_id = contract_id
        self._execution_time = execution_time
        self._video_quality_kind = video_quality_kind
        self._audio_quality_kind = audio_quality_kind
        self._video_quality = video_quality
        self._audio_quality = (
            audio_quality if audio_quality_kind != AudioQualityKind.NONE else None
        )
        self._origin_frame_rate = origin_frame_rate
        self._origin_duration = origin_duration
        self._origin_frame_count = origin_frame_count
        self._output_frame_rate = output_frame_rate
        self._output_duration = output_duration
        self._output_frame_count = output_frame_count

    @property
    def executiontime(self):
        return self._execution_time

    @property
    def contractid(self):
        return self._contract_id

    @property
    def videoqualitykind(self):
        return self._video_quality_kind.value

    @property
    def audioqualitykind(self):
        return self._audio_quality_kind.value

    @property
    def videoquality(self):
        return self._video_quality

    @property
    def audioquality(self):
        return self._audio_quality

    @property
    def origin_frame_rate(self):
        return self._origin_frame_rate

    @property
    def origin_duration(self):
        return self._origin_duration

    @property
    def origin_frame_count(self):
        return self._origin_frame_count

    @property
    def output_frame_rate(self):
        return self._output_frame_rate

    @property
    def output_duration(self):
        return self._output_duration

    @property
    def output_frame_count(self):
        return self._output_frame_count

    def to_dict(self):
        """
        Returns a dictionary representation of the QualityOfServiceMetric.
        """
        return {
            attr: getattr(self, attr)
            for attr in dir(self)
            if attr.startswith("_") and not attr.startswith("__")
        }
