from fastapi import FastAPI

from api.routes import router

app = FastAPI()

app.include_router(router)


@app.get("/")
async def health():
    return {
        "status": "online",
        "service": "icha-music-worker",
    }
