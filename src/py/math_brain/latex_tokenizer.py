#!/usr/bin/env python3
"""
Copyright (c) 2019-2020 David Shin.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.

Adapted from Peter Jin's tokenizer.rs
"""
from abc import ABCMeta, abstractmethod
from enum import Enum
import os
import sys
from typing import List, Optional, Union


class Mode(Enum):
    Text = 0
    InlineMath = 1
    DisplayMath = 2


class MathStart(Enum):
    Dollar = 0
    Bracket = 1


class Pos(Enum):
    Begin = 0
    End = 1


def type_str(s):
    s = str(type(s))
    s = s.split("'")[1]
    s = s.replace('__main__.', '')
    s = s.split('.')[-1]
    return s


StrBuf = List[str]
StrBufLike = Union[StrBuf, str, None]


def to_str_buf(s: StrBufLike) -> StrBuf:
    if isinstance(s, str):
        return [s]
    if isinstance(s, list):
        return s
    return []


class LatexToken:
    def __str__(self):
        return f'[{type_str(self)}]'


class Item(metaclass=ABCMeta):
    def __str__(self):
        return f'[{type_str(self)}]'

    @abstractmethod
    def to_toks(self) -> List[LatexToken]:
        pass

    @abstractmethod
    def handle(self, tokenizer: 'LatexTokenizer', c: str) -> Optional[bool]:
        """
        A truthy return value indicates that the parent loop should continue.
        """
        pass


class LatexTokenizer:
    def __init__(self):
        self.stack: List[Item] = [Items.Text('')]
        self.mode: Mode = Mode.Text
        self.toks: List[LatexToken] = []

    def commit_text(self, buf: str):
        if buf:
            self.commit(LatexTokens.Text(buf))

    def commit(self, token: LatexToken):
        # print(f'Commit: {token}')
        assert isinstance(token, LatexToken)
        self.toks.append(token)

    def enqueue(self, item: Item):
        #print(f'Enqueue: {item}')
        self.stack.append(item)

    def pop(self) -> Item:
        return self.stack.pop()

    def tokenize_math_char(self, buf: StrBuf, c: str, item_lambda):
        # print(f'tokenize_math_char({"".join(buf)}, {c})')
        text = ''.join(buf)
        if c in "+-<>.,?;:[]()\\^_{}'":
            self.commit_text(text)
            self.commit(LatexTokens.from_char(c, True))
            self.enqueue(item_lambda(Pos.End))
        elif c.isspace() or c.isnumeric() or c in '\\':
            self.commit_text(text)
            self.enqueue(item_lambda(Pos.End))
            self.commit(LatexTokens.from_char(c, True))
        else:
            self.commit_text(text)
            self.enqueue(item_lambda(Pos.End, c))

    def tokenize_char(self, c: str):
        # print(f'tokenize_char({c})')
        while self.stack:
            item: Item = self.stack.pop()
            # print(f'stack: {item}')
            if not item.handle(self, c):
                break

    def tokenize(self, source: str):
        # print('Reading source')
        for c in source:
            self.tokenize_char(c)
        # print('Popping remaining stack')
        while self.stack:
            item = self.stack.pop()
            # print(f'stack: {item}')
            for tok in item.to_toks():
                self.commit(tok)


class LatexTokenWithText(LatexToken):
    def __init__(self, buf: StrBufLike):
        self.text = ''.join(to_str_buf(buf))

    def __str__(self):
        text = self.text.replace('\n', '\\n')
        return f'[{type_str(self)}:{text}]'


class LatexTokens:
    class Text(LatexTokenWithText): pass
    class Number(LatexTokenWithText): pass
    class Command(LatexTokenWithText): pass
    class Symbol(LatexTokenWithText): pass
    class Punct(LatexTokenWithText): pass

    class LBrack(LatexToken): pass
    class RBrack(LatexToken): pass
    class LCurly(LatexToken): pass
    class RCurly(LatexToken): pass
    class LParen(LatexToken): pass
    class RParen(LatexToken): pass
    class LQuote(LatexToken): pass
    class RQuote(LatexToken): pass
    class LDQuote(LatexToken): pass
    class RDQuote(LatexToken): pass
    class VBar(LatexToken): pass
    class Space(LatexToken): pass
    class Super(LatexToken): pass
    class Sub(LatexToken): pass
    class LGroup(LatexToken): pass
    class RGroup(LatexToken): pass
    class StartInlineMath(LatexToken): pass
    class EndInlineMath(LatexToken): pass
    class StartDisplayMath(LatexToken): pass
    class EndDisplayMath(LatexToken): pass

    punct_chars = '.,?;:'
    symbol_chars = '+-<>'

    char_map = {
        '[': LBrack,
        ']': RBrack,
        '(': LParen,
        ')': RParen,
        '{': LGroup,
        '}': RGroup,
        '`': LQuote,
        "'": RQuote,
        '|': VBar,
    }

    cmd_char_map = dict(char_map)
    cmd_char_map.update({
        '{': LCurly,
        '}': RCurly,
        '^': Super,
        '_': Sub,
    })

    @staticmethod
    def from_char(c: str, cmd=False) -> LatexToken:
        if c in LatexTokens.punct_chars:
            return LatexTokens.Punct(c)
        if c in LatexTokens.symbol_chars:
            return LatexTokens.Symbol(c)
        if c.isspace():
            return LatexTokens.Space()
        if c.isnumeric():
            return LatexTokens.Number(c)
        if c == '\\':
            return LatexTokens.Command(c)
        cmap = LatexTokens.cmd_char_map if cmd else LatexTokens.char_map
        return cmap[c]()


class TextPushableItem(Item, metaclass=ABCMeta):
    def __init__(self, s: StrBufLike=None):
        self.buf: StrBuf = to_str_buf(s)

    def __str__(self):
        return f'[{type_str(self)}:{"".join(self.buf)}]'

    def push_to(self, tokenizer: LatexTokenizer):
        if not self.buf: return
        tokenizer.commit(LatexTokens.Text(self.buf))


class Items:
    class Text(TextPushableItem):
        def to_toks(self):
            return [LatexTokens.Text(self.buf)] if self.buf else []

        def handle(self, tokenizer: LatexTokenizer, c: str):
            tokenizer.mode = Mode.Text
            if c == '$':
                self.push_to(tokenizer)
                tokenizer.enqueue(Items.Text())
                tokenizer.enqueue(Items.InlineMath(MathStart.Dollar, Pos.Begin))
            elif c in '.,?;:()':
                self.push_to(tokenizer)
                tokenizer.commit(LatexTokens.from_char(c))
                tokenizer.enqueue(Items.Text())
            elif c in "`'":
                self.push_to(tokenizer)
                tokenizer.enqueue(Items.Text())
                tokenizer.enqueue(Items.from_char(c))
            elif c == '{':
                self.push_to(tokenizer)
                tokenizer.commit(LatexTokens.LGroup())
                tokenizer.enqueue(Items.Text())
            elif c == '}':
                self.push_to(tokenizer)
                tokenizer.commit(LatexTokens.RGroup())
                tokenizer.enqueue(Items.Text())
            else:
                self.buf.append(c)
                tokenizer.enqueue(self)

    class LQuote(Item):
        def to_toks(self):
            return [LatexTokens.LQuote()]

        def handle(self, tokenizer: LatexTokenizer, c: str):
            if c == '`':
                tokenizer.commit(LatexTokens.LDQuote())
            else:
                tokenizer.commit(LatexTokens.LQuote())
                return True

    class RQuote(Item):
        def to_toks(self):
            return [LatexTokens.RQuote()]

        def handle(self, tokenizer: LatexTokenizer, c: str):
            if c == "'":
                tokenizer.commit(LatexTokens.RDQuote())
            else:
                tokenizer.commit(LatexTokens.RQuote())
                return True

    class Number(Item):
        def __init__(self):
            self.buf: StrBuf = []

        def to_toks(self):
            return [LatexTokens.Number(self.buf)] if self.buf else []

        def handle(self, tokenizer: LatexTokenizer, c: str):
            if c.isnumeric():
                self.buf.append(c)
                tokenizer.enqueue(self)
            else:
                assert self.buf, 'bug'
                tokenizer.commit(LatexTokens.Number(self.buf))
                return True

    class Command(TextPushableItem):
        def to_toks(self):
            return [LatexTokens.Command(self.buf)] if self.buf else []

        def handle(self, tokenizer: LatexTokenizer, c: str):
            if not self.buf and c == '[':
                item: Item = tokenizer.pop()
                assert type(item) == Items.Text
                item.push_to(tokenizer)
                tokenizer.commit(LatexTokens.StartInlineMath())
                tokenizer.enqueue(Items.Text())
                tokenizer.enqueue(Items.InlineMath(MathStart.Bracket, Pos.End))
            elif not self.buf and c == ']':
                item: Item = tokenizer.pop()
                assert type(item) == Items.InlineMath
                assert item.start == MathStart.Bracket
                self.push_to(tokenizer)
                tokenizer.commit(LatexTokens.EndInlineMath())
            elif not self.buf and c in '{}|':
                tokenizer.commit(LatexTokens.from_char(c, cmd=True))
            elif c.isalpha():
                self.buf.append(c)
                tokenizer.enqueue(Items.Command(self.buf))
            else:
                assert self.buf
                tokenizer.commit(LatexTokens.Command(self.buf))
                return True

    class Space(Item):
        def to_toks(self):
            return [LatexTokens.Space()]

        def handle(self, tokenizer: LatexTokenizer, c: str):
            if c.isspace():
                tokenizer.enqueue(self)
            else:
                if tokenizer.mode == Mode.Text:
                    tokenizer.commit(LatexTokens.Space())
                return True

    class InlineMath(TextPushableItem):
        def __init__(self, start: MathStart, pos: Pos, s: StrBufLike=None):
            self.start = start
            self.pos = pos
            super().__init__(s)

        @staticmethod
        def lambda_init(start: MathStart):
            return lambda *args: Items.InlineMath(start, *args)

        def to_toks(self):
            raise Exception(f'Unterminated InlineMath({self.start})')

        def handle(self, tokenizer: LatexTokenizer, c: str):
            tokenizer.mode = Mode.InlineMath
            if self.pos == Pos.Begin:
                if c == '$':
                    assert self.start == MathStart.Dollar
                    assert not self.buf
                    tokenizer.enqueue(Items.DisplayMath(Pos.Begin))
                    return True
                else:
                    tokenizer.commit(LatexTokens.StartInlineMath())
            if c == '$':
                assert self.start == MathStart.Dollar
                self.push_to(tokenizer)
                tokenizer.commit(LatexTokens.EndInlineMath())
            else:
                tokenizer.tokenize_math_char(self.buf, c, Items.InlineMath.lambda_init(self.start))

    class DisplayMath(TextPushableItem):
        def __init__(self, pos: Pos, s: StrBufLike=None):
            self.pos = pos
            super().__init__(s)

        def __str__(self):
            return f'[{type_str(self)}:{self.pos}]'

        def to_toks(self):
            raise Exception('Unterminated DisplayMath')

        def handle(self, tokenizer: LatexTokenizer, c: str):
            tokenizer.mode = Mode.DisplayMath
            if self.pos == Pos.End:
                assert c != '$'
                tokenizer.commit(LatexTokens.EndDisplayMath())
                return True
            if c == '$':
                tokenizer.enqueue(Items.DisplayMath(Pos.End, self.buf))
            else:
                tokenizer.tokenize_math_char(self.buf, c, Items.DisplayMath)

    char_map = {
        '`': LQuote,
        "'": RQuote,
    }

    @staticmethod
    def from_char(c: str) -> Item:
        if c == '\\':
            return Items.Command(c)
        if c.isspace():
            return Items.Space()
        return Items.char_map[c]()


class LatexDocument:
    def __init__(self, text: str):
        self.text = text
        tokenizer = LatexTokenizer()
        tokenizer.tokenize(text)
        self.toks = tokenizer.toks


if __name__ == '__main__':
    filename = os.path.expanduser(sys.argv[1])
    with open(filename) as f:
        text = f.read()
    doc = LatexDocument(text)
    print('\n'.join(map(str, doc.toks)))

