from typing import Dict, List, Optional
import xml.etree.ElementTree as ET


def elem_to_str(elem: ET.Element) -> str:
    return ET.tostring(elem).decode('utf-8')


def get_inside_text(element: ET.Element) -> Optional[str]:
    """
    Returns everything between <foo> and closing </foo>.

    Note that ET.tostring() works if you want the tags as well.

    Adapted from: https://stackoverflow.com/a/381614/543913
    """
    tokens = [element.text or '']
    tokens.extend(map(elem_to_str, element))
    tokens.append(element.tail or '')
    return ''.join(tokens).strip()


def get_node(
        tree: ET.Element,
        tag: Optional[str]=None,
        attribs: Optional[Dict[str, str]]=None,
        index: Optional[int]=None
        ) -> List[ET.Element]:
    """
    Finds the top-level nodes of <tree> which match <tag> and whose attrib
    mappings contains <attribs>.

    If index is None, asserts that only one such node matches, and returns it.
    Otherwise, returns the <index>'th node.
    """
    if attribs is None:
        attribs = {}

    nodes = [node for node in list(tree) if node.tag == tag and
            all([node.attrib.get(k, None)==v for k, v in attribs.items()])]
    if index is None:
        assert len(nodes)==1
        return nodes[0]
    return nodes[index]

