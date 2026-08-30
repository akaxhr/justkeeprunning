import os
import asyncio

from fastapi import FastAPI, Header, HTTPException
from pydantic import BaseModel
import uvicorn

from pyrogram import Client
from pytgcalls import PyTgCalls


# =========================
# ENVIRONMENT
# =========================

API_ID = int(os.environ["API_ID"])
API_HASH = os.environ["API_HASH"]
SESSION_STRING = os.environ["SESSION_STRING"]

WORKER_SECRET = os.environ["WORKER_SECRET"]


# =========================
# TELEGRAM MUSIC ACCOUNT
# =========================

app_telegram = Client(
    "icha_music",
    api_id=API_ID,
    api_hash=API_HASH,
    session_string=SESSION_STRING,
)

calls = PyTgCalls(app_telegram)


# =========================
# HTTP API
# =========================

app = FastAPI()


class PlayRequest(BaseModel):
    chat_id: int
    query: str


def check_secret(authorization):
    if authorization != f"Bearer {WORKER_SECRET}":
        raise HTTPException(status_code=401, detail="Unauthorized")


@app.get("/")
async def health():
    return {
        "status": "online",
        "service": "icha-music-worker",
    }


@app.post("/play")
async def play(
    request: PlayRequest,
    authorization: str = Header(None),
):
    check_secret(authorization)

    print(
        f"Play request received: "
        f"chat={request.chat_id}, query={request.query}"
    )

    return {
        "status": "received",
        "chat_id": request.chat_id,
        "query": request.query,
    }


# =========================
# START EVERYTHING
# =========================

async def telegram_worker():

    await app_telegram.start()
    await calls.start()

    me = await app_telegram.get_me()

    print(
        f"Music account logged in: "
        f"{me.first_name} (@{me.username})"
    )

    print("Icha Music Worker is running.")

    await asyncio.Event().wait()


async def main():

    telegram_task = asyncio.create_task(
        telegram_worker()
    )

    config = uvicorn.Config(
        app,
        host="0.0.0.0",
        port=int(os.environ.get("PORT", 8000)),
        log_level="info",
    )

    server = uvicorn.Server(config)

    await server.serve()

    await telegram_task


if __name__ == "__main__":
    asyncio.run(main())
