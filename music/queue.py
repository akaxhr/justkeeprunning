from collections import deque


class MusicQueue:

    def __init__(self):
        self._queues = {}

    def _get_queue(self, chat_id):

        if chat_id not in self._queues:
            self._queues[chat_id] = deque()

        return self._queues[chat_id]

    def add(self, chat_id, track):

        queue = self._get_queue(chat_id)
        queue.append(track)

    def pop(self, chat_id):

        queue = self._get_queue(chat_id)

        if not queue:
            return None

        return queue.popleft()

    def peek(self, chat_id):

        queue = self._get_queue(chat_id)

        if not queue:
            return None

        return queue[0]

    def all(self, chat_id):

        return list(self._get_queue(chat_id))

    def clear(self, chat_id):

        self._get_queue(chat_id).clear()

    def size(self, chat_id):

        return len(self._get_queue(chat_id))


music_queue = MusicQueue()
