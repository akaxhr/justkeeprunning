from pyrogram import Client
from pytgcalls import PyTgCalls

from config.settings import (
    API_ID,
    API_HASH,
    SESSION_STRING,
)


app_telegram = Client(
    "icha_music",
    api_id=API_ID,
    api_hash=API_HASH,
    session_string=SESSION_STRING,
)

calls = PyTgCalls(app_telegram)


async def start_telegram():

    await app_telegram.start()
    await calls.start()

    me = await app_telegram.get_me()

    print(
        f"🎵 Music account logged in: "
        f"{me.first_name} (@{me.username})"
    )

    return me
