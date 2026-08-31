FROM python:3.11-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ffmpeg \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY requirements.txt .

RUN pip install --no-cache-dir -r requirements.txt

RUN python -c "from pathlib import Path; p=Path('/usr/local/lib/python3.11/site-packages/pytgcalls/mtproto/pyrogram_client.py'); s=p.read_text(); s=s.replace('from pyrogram.errors import GroupcallForbidden', 'try:\n    from pyrogram.errors import GroupcallForbidden\nexcept ImportError:\n    class GroupcallForbidden(Exception):\n        pass'); p.write_text(s)"

COPY . .

CMD ["python", "-u", "main.py"]
