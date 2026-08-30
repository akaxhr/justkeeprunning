class MusicQueue:

    def __init__(self):
        self.queues = {}

    def _get(self, chat_id):
        return self.queues.setdefault(chat_id, [])

    def add(self, chat_id, song):
        queue = self._get(chat_id)
        queue.append(song)
        return len(queue)

    def pop(self, chat_id):
        queue = self._get(chat_id)

        if not queue:
            return None

        return queue.pop(0)

    def peek(self, chat_id):
        queue = self._get(chat_id)

        if not queue:
            return None

        return queue[0]

    def get_all(self, chat_id):
        return self._get(chat_id)

    def clear(self, chat_id):
        self.queues[chat_id] = []

    def remove(self, chat_id, index):
        queue = self._get(chat_id)

        if index < 1 or index > len(queue):
            return None

        return queue.pop(index - 1)

    def size(self, chat_id):
        return len(self._get(chat_id))


music_queue = MusicQueue()
