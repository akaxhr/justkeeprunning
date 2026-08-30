from fastapi import FastAPI, Header, HTTPException
from pydantic import BaseModel
import os

from telegram.client import app_telegram
from telegram.voice import calls
from music.player import play_song


app = FastAPI()


class PlayRequest(BaseModel):
    chat_id: int
    query: str


def check_secret(authorization):
    if authorization != f"Bearer {os.environ['WORKER_SECRET']}":
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

    result = await play_song(
        calls,
        request.chat_id,
        request.query,
    )

    return result
