#!/usr/bin/env python3
"""
Does a best-effort parse of a encyclopediaofmath.org article.
"""
import os
import sys
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
from typing import Dict, List, Optional
import xml.etree.ElementTree as ET

from util.web_cache import WebCache
from util.xml_util import get_inside_text, get_node


class ExtraSection:
    def __init__(self, title):
        self._title = title
        self._lines = []
        self._text = None

    def append(self, line):
        self._lines.append(line)

    def finalize(self):
        self._text = '\n'.join(self._lines)
        self._lines = None

    @property
    def title(self) -> str:
        return self._title

    @property
    def text(self) -> str:
        return self._text

    def dump(self):
        print(f'** {self.title} **')
        print(self.text)


class EncyclopediaOfMathArticle:
    """
    A structured representation of an encyclopediaofmath.org article.

    The implementation of this class takes advantage of the very specific
    format of each article.
    """
    @staticmethod
    def fix_malformed_html(text):
        """
        &nbsp; hack adapted from: https://stackoverflow.com/a/35591507/543913
        """
        text = text.replace('&returnto', '&amp;returnto')
        text = text.replace('&oldid=', '&amp;oldid=')
        magic = '''<!DOCTYPE html [
            <!ENTITY nbsp ' '>
            ]>'''
        return magic + text[text.find('\n'):]

    def __init__(self, html_text: str):
        html_text = EncyclopediaOfMathArticle.fix_malformed_html(html_text)
        tree = ET.fromstring(html_text)
        body = get_node(tree, 'body')
        outer_shell = get_node(body, 'div', {'id': 'OuterShell'})
        inner_shell = get_node(outer_shell, 'div', {'id': 'InnerShell'})
        content_shell = get_node(inner_shell, 'div', {'id': 'ContentShell'})
        content = get_node(content_shell, 'div', {'id': 'content'})
        inner_content = get_node(content, 'div', {'id': 'InnerContent'})

        first_heading = get_node(inner_content, 'h1', {'id': 'firstHeading'})
        heading_text = get_inside_text(first_heading)
        heading_prefix = 'View source for '
        assert heading_text.startswith(heading_prefix), heading_text
        topic = heading_text[len(heading_prefix):]

        body_content = get_node(inner_content, 'div', {'id': 'bodyContent'})
        mw_content_text = get_node(body_content, 'div', {'id': 'mw-content-text'})
        text_area = get_node(mw_content_text, 'textarea', {'id': 'wpTextbox1'})
        complete_text = get_inside_text(text_area)

        # text can contain extra sections that start with ==== (references, comments)
        main_text_lines = []
        extra_sections = []
        for line in complete_text.splitlines():
            if line.startswith('====') and line.endswith('===='):
                section_name = line[4:-4]
                assert section_name in ('References', 'Comments'), line
                section = ExtraSection(section_name)
                extra_sections.append(section)
                continue
            if extra_sections:
                extra_sections[-1].append(line)
            else:
                main_text_lines.append(line)

        for section in extra_sections:
            section.finalize()

        self._topic = topic
        self._main_text = '\n'.join(main_text_lines)
        self._extra_sections = extra_sections

    @property
    def topic(self) -> str:
        return self._topic

    @property
    def main_text(self) -> str:
        return self._main_text

    @property
    def extra_sections(self) -> List[ExtraSection]:
        return self._extra_sections

    def dump(self, include_extra_sections=False):
        print(f'* {self.topic} *')
        print('')
        print(self.main_text)
        if include_extra_sections:
            for section in self.extra_sections:
                print('')
                section.dump()


def to_edit_page(url: str) -> str:
    assert url.find('encyclopediaofmath.org') != -1
    tokens = url.split('/')
    wiki_indices = [i for (i, t) in enumerate(tokens) if t == 'wiki']
    w = len(wiki_indices)
    if w == 0:
        assert url.find('index.php') != -1, url
        assert url.find('action=edit') != -1, url
        return url
    assert w == 1, url
    wi = wiki_indices[0]
    assert wi+2 == len(tokens), url
    topic = tokens[-1]
    return f'https://encyclopediaofmath.org/index.php?title={topic}&action=edit'


class EncyclopediaOfMathParser:
    def __init__(self, cache: Optional[WebCache]=None):
        self._cache = cache
        if cache is None:
            self._cache = WebCache()

    def parse(self, url) -> EncyclopediaOfMathArticle:
        """
        Example url: https://encyclopediaofmath.org/wiki/Triangle

        The edit page contains the latex source.
        """
        url = to_edit_page(url)
        text = self._cache.html_request(url)
        return EncyclopediaOfMathArticle(text)


def main():
    if len(sys.argv) != 2:
        script = os.path.basename(__file__)
        print(f'Usage: {script} <TOPIC>')
        print(f'Example: {script} circle')
        pass

    topic = sys.argv[1].lower()
    parser = EncyclopediaOfMathParser()
    if not topic.startswith('http'):
        url = f'https://encyclopediaofmath.org/wiki/{topic}'
    else:
        url = topic
    parser.parse(url).dump()


if __name__ == '__main__':
    main()


