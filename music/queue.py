class MusicQueue:

    def __init__(self):
        self.data = {}

    def get(self, chat_id):
        return self.data.setdefault(chat_id, [])

    def add(self, chat_id, song):
        queue = self.get(chat_id)
        queue.append(song)
        return len(queue)

    def next(self, chat_id):
        queue = self.get(chat_id)
        return queue.pop(0) if queue else None

    def clear(self, chat_id):
        self.data[chat_id] = []

    def all(self, chat_id):
        return self.get(chat_id)


queue = MusicQueue()
