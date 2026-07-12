import os
import re
import asyncio

from telethon import TelegramClient, events
from telethon.sessions import StringSession


API_ID = int(os.environ["TELEGRAM_API_ID"])
API_HASH = os.environ["TELEGRAM_API_HASH"]
SESSION_STRING = os.environ["TELEGRAM_SESSION_STRING"]

TARGET_GROUP_ID = int(os.environ["TARGET_GROUP_ID"])

# Without @, for example: SomeGameBot
GAME_BOT_USERNAME = os.environ["GAME_BOT_USERNAME"].lstrip("@").lower()


client = TelegramClient(
    StringSession(SESSION_STRING),
    API_ID,
    API_HASH,
)


@client.on(events.NewMessage(chats=TARGET_GROUP_ID))
async def auto_answer(event):
    if event.out:
        return

    sender = await event.get_sender()
    sender_username = (getattr(sender, "username", None) or "").lower()

    if sender_username != GAME_BOT_USERNAME:
        return

    text = event.raw_text or ""

    match = re.search(
        r"(?:^|\n)\s*(?:🔤\s*)?Word:\s*([A-Za-z]+)",
        text,
        flags=re.IGNORECASE,
    )

    if not match:
        return

    word = match.group(1)

    await asyncio.sleep(0.1)
    await client.send_message(event.chat_id, word)

    print(f"Answered in {event.chat_id}: {word}")


async def main():
    await client.start()
    print("Telegram word auto-reply is running")
    await client.run_until_disconnected()


if __name__ == "__main__":
    asyncio.run(main())