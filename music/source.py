import yt_dlp


async def get_audio(query):
    loop = __import__("asyncio").get_running_loop()

    def search():
        opts = {
            "format": "bestaudio/best",
            "quiet": True,
            "default_search": "ytsearch",
            "noplaylist": True,
        }

        with yt_dlp.YoutubeDL(opts) as ydl:
            info = ydl.extract_info(query, download=False)

        return info["entries"][0]["url"]

    return await loop.run_in_executor(None, search)
