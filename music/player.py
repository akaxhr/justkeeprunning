import os

from pytgcalls import PyTgCalls
from pytgcalls.types import AudioPiped

from telegram.voice import calls

from music.queue import music_queue
from music.search import search_song
from music.downloader import download_audio


class MusicPlayer:

    def __init__(self):
        self.current = {}

    async def play(self, chat_id, query, quality="high"):

        print(
            f"🎵 Play request | chat={chat_id} | query={query}"
        )

        song = search_song(query)

        if not song:
            return {
                "status": "error",
                "message": "Song not found",
            }

        song["requested_query"] = query

        # If something is already playing, add to queue
        if chat_id in self.current:

            position = music_queue.add(
                chat_id,
                song,
            )

            return {
                "status": "queued",
                "title": song["title"],
                "position": position,
            }

        return await self._start_song(
            chat_id,
            song,
            quality,
        )

    async def _start_song(
        self,
        chat_id,
        song,
        quality="high",
    ):

        print(f"⬇️ Preparing: {song['title']}")

        try:

            audio_file = download_audio(
                song["webpage_url"],
                quality,
            )

            print(f"▶️ Starting: {song['title']}")

            await calls.play(
                chat_id,
                AudioPiped(audio_file),
            )

            self.current[chat_id] = {
                **song,
                "file": audio_file,
                "quality": quality,
            }

            return {
                "status": "playing",
                "title": song["title"],
            }

        except Exception as e:

            print(
                f"❌ Playback error: {type(e).__name__}: {e}"
            )

            return {
                "status": "error",
                "message": str(e),
            }

    async def skip(self, chat_id):

        current = self.current.pop(
            chat_id,
            None,
        )

        if current:
            self._delete_file(current)

        next_song = music_queue.pop(chat_id)

        if not next_song:

            try:
                await calls.leave_call(chat_id)
            except Exception:
                pass

            return {
                "status": "stopped",
            }

        return await self._start_song(
            chat_id,
            next_song,
        )

    async def stop(self, chat_id):

        music_queue.clear(chat_id)

        current = self.current.pop(
            chat_id,
            None,
        )

        if current:
            self._delete_file(current)

        try:
            await calls.leave_call(chat_id)
        except Exception:
            pass

        return {
            "status": "stopped",
        }

    def get_current(self, chat_id):
        return self.current.get(chat_id)

    def get_queue(self, chat_id):
        return music_queue.get_all(chat_id)

    def _delete_file(self, song):

        filename = song.get("file")

        if filename and os.path.exists(filename):

            try:
                os.remove(filename)
            except Exception:
                pass


player = MusicPlayer()
