from typing import Optional

from pytgcalls.types import AudioPiped
from pytgcalls import PyTgCalls

from music.models import Track


class MusicPlayer:

    def __init__(self, calls: PyTgCalls):
        self.calls = calls

        # Current track for each Telegram group
        self.current_tracks: dict[int, Track] = {}

        # Playback state
        self.paused: set[int] = set()

    async def play(
        self,
        chat_id: int,
        track: Track,
    ):

        await self.calls.play(
            chat_id,
            AudioPiped(track.file_path),
        )

        self.current_tracks[chat_id] = track
        self.paused.discard(chat_id)

        print(
            f"▶️ Playing '{track.title}' "
            f"in {chat_id}"
        )

    async def pause(self, chat_id: int):

        await self.calls.pause(chat_id)

        self.paused.add(chat_id)

        print(f"⏸ Paused {chat_id}")

    async def resume(self, chat_id: int):

        await self.calls.resume(chat_id)

        self.paused.discard(chat_id)

        print(f"▶️ Resumed {chat_id}")

    async def stop(self, chat_id: int):

        await self.calls.leave_call(chat_id)

        self.current_tracks.pop(chat_id, None)
        self.paused.discard(chat_id)

        print(f"⏹ Stopped {chat_id}")

    async def skip(self, chat_id: int):

        # For now skip simply stops the current track.
        # Queue integration will be added next.
        await self.stop(chat_id)

        print(f"⏭ Skipped {chat_id}")

    def current(self, chat_id: int) -> Optional[Track]:

        return self.current_tracks.get(chat_id)

    def is_paused(self, chat_id: int) -> bool:

        return chat_id in self.paused
