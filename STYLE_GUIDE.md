# Contributors Style Guide

(Status: Tentative and subject to change. Feedback welcome.)

## General text format

Contributed text files containing questions/problems and solutions may be
thought of as having two parts: (1) a header and (2) the content.

### Header

The header is at the beginning of the text file and consists of one or more
consecutive lines. A header line must begin with two percent ('%') characters.

A header line is a key-value pair, where the key and value are separated by a
colon (':') character.

The very first header line may consist of only a value without a key; the value
would then be the title of the text file. Alternatively, the first header line
may also be a key-value pair, in which case the value is a reference to the
title of another text file in the repository, and the key is a label.

### Content

The content should follow the header. The header and content should be
separated by an empty line. The mathematical part of the content should
primarily be LaTeX. It is suggested that lines are limited in width to a
standard character count; e.g. 80, 100, or 120 characters per line.

### Examples

Example 1:

    %% FOMO 1999 Q42
    %% transcribed by: Foobar Baz
    %% original author: FOMO Committee

    Let $u$ be eaten by a \emph{grue}.

Example 2:

    %% solution to: FOMO 1999 Q42
    %% author: Alice Bob

    This was a triumph.
