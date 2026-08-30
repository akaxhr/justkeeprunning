from pytgcalls.types import AudioPiped

from telegram.voice import calls


async def play_audio(chat_id, file_path):
    await calls.play(
        chat_id,
        AudioPiped(file_path),
    )


async def stop_audio(chat_id):
    try:
        await calls.leave_call(chat_id)
    except Exception:
        pass
