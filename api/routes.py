from fastapi import APIRouter, Header, HTTPException
from pydantic import BaseModel

from config.settings import WORKER_SECRET
from music.player import player

router = APIRouter()


class PlayRequest(BaseModel):
    chat_id: int
    query: str


def check_secret(authorization):
    if authorization != f"Bearer {WORKER_SECRET}":
        raise HTTPException(status_code=401)


@router.post("/play")
async def play(
    request: PlayRequest,
    authorization: str = Header(None),
):
    check_secret(authorization)

    return await player.play(
        request.chat_id,
        request.query,
    )
