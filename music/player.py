from pytgcalls import PyTgCalls
from pytgcalls.types import AudioPiped
from pytgcalls import idle

from .source import get_audio


async def play_song(client, calls, chat_id, query):
    audio = await get_audio(query)

    await calls.play(
        chat_id,
        AudioPiped(audio)
    )

    return audio
