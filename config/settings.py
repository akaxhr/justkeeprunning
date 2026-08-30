import os


API_ID = int(os.environ["API_ID"])
API_HASH = os.environ["API_HASH"]
SESSION_STRING = os.environ["SESSION_STRING"]

WORKER_SECRET = os.environ["WORKER_SECRET"]

PORT = int(os.environ.get("PORT", 8000))
