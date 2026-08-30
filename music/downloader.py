import asyncio
import os
import re
import uuid

from music.models import Track


DOWNLOAD_DIR = "/tmp/icha_music"

os.makedirs(DOWNLOAD_DIR, exist_ok=True)


def sanitize_filename(name: str) -> str:
    name = re.sub(r'[\\/*?:"<>|]', "", name)
    return name[:100].strip()


async def download_song(
    query: str,
    requester_id: int | None = None,
    requester_name: str | None = None,
) -> Track:

    query = query.strip()

    if not query:
        raise ValueError("Song query cannot be empty")

    job_id = uuid.uuid4().hex

    output_template = os.path.join(
        DOWNLOAD_DIR,
        f"{job_id}.%(ext)s",
    )

    command = [
        "yt-dlp",
        "--no-playlist",
        "--extract-audio",
        "--audio-format",
        "opus",
        "--audio-quality",
        "128K",
        "--print",
        "title",
        "--print",
        "duration",
        "--print",
        "webpage_url",
        "-o",
        output_template,
        f"ytsearch1:{query}",
    ]

    process = await asyncio.create_subprocess_exec(
        *command,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )

    stdout, stderr = await process.communicate()

    if process.returncode != 0:
        error = stderr.decode(errors="ignore")

        print(f"❌ yt-dlp error:\n{error}")

        raise RuntimeError(
            "Unable to find or download that song."
        )

    lines = stdout.decode(
        errors="ignore"
    ).strip().splitlines()

    if len(lines) < 3:
        raise RuntimeError(
            "Could not read downloaded song information."
        )

    title = lines[0].strip()

    try:
        duration = int(float(lines[1]))
    except ValueError:
        duration = 0

    url = lines[2].strip()

    files = [
        os.path.join(DOWNLOAD_DIR, file)
        for file in os.listdir(DOWNLOAD_DIR)
        if file.startswith(job_id + ".")
    ]

    if not files:
        raise RuntimeError(
            "Song downloaded but audio file was not found."
        )

    file_path = files[0]

    track = Track(
        title=title,
        duration=duration,
        file_path=file_path,
        requester_id=requester_id,
        requester_name=requester_name,
        source="youtube",
        url=url,
    )

    print(
        f"🎧 Downloaded: {track.title} "
        f"({track.duration}s)"
    )

    return track
