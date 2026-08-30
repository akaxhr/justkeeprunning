from pytgcalls.types import AudioPiped

from .source import get_audio
from .queue import add, get_next


async def play_song(calls, chat_id, query):
    song = await get_audio(query)

    if chat_id in calls.active_calls:
        add(chat_id, song)

        return {
            "status": "queued",
            "song": song,
        }

    await calls.play(
        chat_id,
        AudioPiped(song["url"]),
    )

    return {
        "status": "playing",
        "song": song,
    }


async def play_next(calls, chat_id):
    song = get_next(chat_id)

    if not song:
        return None

    await calls.play(
        chat_id,
        AudioPiped(song["url"]),
    )

    return song
