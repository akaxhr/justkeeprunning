import asyncio
import uvicorn

from telegram.client import app_telegram
from telegram.voice import start_voice
from api.server import app


async def telegram_worker():

    await app_telegram.start()

    me = await app_telegram.get_me()

    print(
        f"Music account logged in: "
        f"{me.first_name} (@{me.username})"
    )

    await start_voice()

    print("🎵 Icha Music Worker is running.")

    await asyncio.Event().wait()


async def main():

    telegram_task = asyncio.create_task(
        telegram_worker()
    )

    config = uvicorn.Config(
        app,
        host="0.0.0.0",
        port=int(__import__("os").environ.get("PORT", 8000)),
        log_level="info",
    )

    server = uvicorn.Server(config)

    await server.serve()

    await telegram_task


if __name__ == "__main__":
    asyncio.run(main())
