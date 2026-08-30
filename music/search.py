import yt_dlp


def search_song(query: str):
    options = {
        "quiet": True,
        "no_warnings": True,
        "default_search": "ytsearch1",
        "extract_flat": True,
    }

    with yt_dlp.YoutubeDL(options) as ydl:
        info = ydl.extract_info(query, download=False)

    if not info or not info.get("entries"):
        return None

    result = info["entries"][0]

    return {
        "title": result.get("title"),
        "url": result.get("url"),
        "webpage_url": result.get("webpage_url"),
        "duration": result.get("duration"),
    }
