char = str


def char_repr(c: char):
    c = r'\n' if c=='\n' else c
    return f'"{c}"'


def type_str(obj):
    s = str(type(obj))
    s = s.split("'")[1]
    s = s.replace('__main__.', '')
    s = s.split('.')[-1]
    return s

