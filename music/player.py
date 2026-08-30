from pytgcalls.types import AudioPiped

from .source import get_audio
from .queue import add, get_next


async def play_song(calls, chat_id, query):

    audio = await get_audio(query)

    song = {
        "query": query,
        "audio": audio,
    }

    # If something is already playing, queue it
    if chat_id in calls.active_calls:
        add(chat_id, song)

        return {
            "status": "queued",
            "query": query,
        }

    await calls.play(
        chat_id,
        AudioPiped(audio),
    )

    return {
        "status": "playing",
        "query": query,
    }


async def play_next(calls, chat_id):

    song = get_next(chat_id)

    if not song:
        return None

    await calls.play(
        chat_id,
        AudioPiped(song["audio"]),
    )

    return song
