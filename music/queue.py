from collections import defaultdict, deque

queues = defaultdict(deque)


def add(chat_id, song):
    queues[chat_id].append(song)


def get_next(chat_id):
    if not queues[chat_id]:
        return None

    return queues[chat_id].popleft()


def get_queue(chat_id):
    return list(queues[chat_id])


def clear(chat_id):
    queues[chat_id].clear()
