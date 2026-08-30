import os
import asyncio

from pyrogram import Client
from pytgcalls import PyTgCalls

API_ID = int(os.environ["API_ID"])
API_HASH = os.environ["API_HASH"]
SESSION_STRING = os.environ["SESSION_STRING"]

app = Client(
    "icha_music",
    api_id=API_ID,
    api_hash=API_HASH,
    session_string=SESSION_STRING,
)

calls = PyTgCalls(app)


async def main():
    await app.start()
    await calls.start()

    me = await app.get_me()

    print(f"Music account logged in: {me.first_name}")
    print("Icha Music Worker is running.")

    await asyncio.Event().wait()


if __name__ == "__main__":
    asyncio.run(main())
