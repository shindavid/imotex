"""
Module to access a local cache of web content.
"""
import os
import re
from typing import Optional
import urllib.request

DEFAULT_WEB_CACHE_DIRECTORY = os.path.expanduser(f'~/mathbrain/web_cache')


class WebCache:
    def __init__(self,
            cache_dir: Optional[str]=DEFAULT_WEB_CACHE_DIRECTORY,
            mode: str='rw'):
        """
        mode 'r': means read cache
        mode 'w': means write cache
        """
        self._cache_dir = cache_dir
        if not os.path.isdir(cache_dir):
            os.makedirs(cache_dir)
        for m in mode:
            assert m in 'rw', f'Invalid mode: "{mode}"'
        self._read_mode = 'r' in mode
        self._write_mode = 'w' in mode
        self._http_regex = re.compile(r"https?://(www\.)?")

    def html_request(self, url: str) -> str:
        """
        Takes a url and returns the url content.
        """
        url = url.lower()  # to normalize cache
        stripped_url = self._http_regex.sub('', url).strip().strip('/')
        cache_file = os.path.join(self._cache_dir, stripped_url)
        if self._read_mode and os.path.isfile(cache_file):
            with open(cache_file) as f:
                return f.read()
        with urllib.request.urlopen(url) as u:
            text = u.read().decode('utf-8')
            if self._write_mode:
                cache_file_dir = os.path.split(cache_file)[0]
                os.makedirs(cache_file_dir, exist_ok=True)
                with open(cache_file, 'w') as f:
                    f.write(text)
        return text

