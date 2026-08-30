from music.queue import queue
from music.search import search_song
from music.downloader import download_audio
from music.playback import play_audio, stop_audio


class Player:

    def __init__(self):
        self.current = {}

    async def play(self, chat_id, query, quality="high"):

        song = search_song(query)

        if not song:
            return {
                "status": "error",
                "message": "Song not found",
            }

        if chat_id in self.current:
            position = queue.add(chat_id, song)

            return {
                "status": "queued",
                "title": song["title"],
                "position": position,
            }

        return await self.start(chat_id, song, quality)

    async def start(self, chat_id, song, quality="high"):

        file_path = download_audio(
            song["webpage_url"],
            quality,
        )

        await play_audio(
            chat_id,
            file_path,
        )

        self.current[chat_id] = {
            **song,
            "file": file_path,
            "quality": quality,
        }

        return {
            "status": "playing",
            "title": song["title"],
        }

    async def skip(self, chat_id):

        self.current.pop(chat_id, None)

        song = queue.next(chat_id)

        if not song:
            await stop_audio(chat_id)
            return {"status": "stopped"}

        return await self.start(chat_id, song)

    async def stop(self, chat_id):

        queue.clear(chat_id)
        self.current.pop(chat_id, None)

        await stop_audio(chat_id)

        return {"status": "stopped"}

    def current_song(self, chat_id):
        return self.current.get(chat_id)

    def get_queue(self, chat_id):
        return queue.all(chat_id)


player = Player()
