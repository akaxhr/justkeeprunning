QUALITY = {
    "low": 64,
    "medium": 128,
    "high": 192,
    "very_high": 320,
}


def get_bitrate(quality="high"):
    return QUALITY.get(quality, 192)
