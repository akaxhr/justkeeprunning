from dataclasses import dataclass
from typing import Optional


@dataclass
class Track:

    title: str
    duration: int
    file_path: str

    requester_id: Optional[int] = None
    requester_name: Optional[str] = None

    source: Optional[str] = None
    thumbnail: Optional[str] = None

    url: Optional[str] = None
