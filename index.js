import { TelegramClient } from "telegram";
import { StringSession } from "telegram/sessions";

const apiId = Number(process.env.API_ID);
const apiHash = process.env.API_HASH;
const session = process.env.SESSION_STRING;

if (!apiId || !apiHash || !session) {
  throw new Error("Missing API_ID, API_HASH or SESSION_STRING");
}

const client = new TelegramClient(
  new StringSession(session),
  apiId,
  apiHash,
  {
    connectionRetries: 10,
  }
);

await client.connect();

const me = await client.getMe();

console.log("=================================");
console.log("🎵 Icha Music Worker");
console.log(`Logged in as: ${me.firstName}`);
console.log(`Username: @${me.username || "none"}`);
console.log("Telegram connection successful.");
console.log("=================================");

process.stdin.resume();
