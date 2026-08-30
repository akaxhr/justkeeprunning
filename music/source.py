import asyncio
import yt_dlp


def _search_youtube(query):
    options = {
        "format": "bestaudio/best",
        "quiet": True,
        "no_warnings": True,
        "default_search": "ytsearch1",
        "noplaylist": True,
    }

    with yt_dlp.YoutubeDL(options) as ydl:
        info = ydl.extract_info(query, download=False)

    if not info:
        raise Exception("No result found")

    if "entries" in info:
        info = info["entries"][0]

    return {
        "title": info.get("title", "Unknown"),
        "url": info["url"],
        "duration": info.get("duration", 0),
        "thumbnail": info.get("thumbnail"),
    }


async def get_audio(query):
    loop = asyncio.get_running_loop()

    return await loop.run_in_executor(
        None,
        _search_youtube,
        query,
    )
