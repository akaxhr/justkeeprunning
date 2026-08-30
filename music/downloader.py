import yt_dlp


def download_audio(url: str, quality="high"):

    bitrate = {
        "low": "64",
        "medium": "128",
        "high": "192",
        "very_high": "320",
    }.get(quality, "192")

    options = {
        "format": "bestaudio/best",
        "outtmpl": "/tmp/%(id)s.%(ext)s",
        "quiet": True,
        "no_warnings": True,
        "postprocessors": [
            {
                "key": "FFmpegExtractAudio",
                "preferredcodec": "opus",
                "preferredquality": bitrate,
            }
        ],
    }

    with yt_dlp.YoutubeDL(options) as ydl:
        info = ydl.extract_info(url, download=True)

        filename = ydl.prepare_filename(info)
        filename = filename.rsplit(".", 1)[0] + ".opus"

    return filename
