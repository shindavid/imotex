from typing import Optional


def ellipsize(s: str, n: Optional[int]=None) -> str:
    if n is None or len(s) <= n:
        return s
    return f'{s[:n-3]}...'


def get_html_header_level(tag: str) -> Optional[int]:
    """
    'h1' -> 1
    """
    if tag[0] == 'h' and tag[1:].isdigit():
        return int(tag[1:])
    return None

