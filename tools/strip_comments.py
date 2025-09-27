#!/usr/bin/env python3
"""
Strip // comments from a Rust source file while preserving block comments (/* */)
and string/char literals. Makes a backup copy before modifying.
Usage:
    python strip_comments.py path/to/main.rs
"""
import sys
import re
from pathlib import Path

if len(sys.argv) < 2:
    print("Usage: strip_comments.py <file>")
    sys.exit(1)

p = Path(sys.argv[1])
if not p.exists():
    print(f"File not found: {p}")
    sys.exit(2)

backup = p.with_suffix(p.suffix + ".bak")
print(f"Creating backup: {backup}")
backup.write_bytes(p.read_bytes())

s = p.read_text(encoding='utf-8')

# We will parse the file character by character to avoid removing // inside strings or block comments.
out_chars = []
state = 'normal'  # normal, line_comment, block_comment, string, char
i = 0
length = len(s)
while i < length:
    c = s[i]
    if state == 'normal':
        if s.startswith('//', i):
            state = 'line_comment'
            i += 2
            continue
        elif s.startswith('/*', i):
            state = 'block_comment'
            out_chars.append('/*')
            i += 2
            continue
        elif c == '"':
            state = 'string'
            out_chars.append(c)
            i += 1
            continue
        elif c == "'":
            state = 'char'
            out_chars.append(c)
            i += 1
            continue
        else:
            out_chars.append(c)
            i += 1
            continue
    elif state == 'line_comment':
        if c == '\n':
            out_chars.append(c)
            state = 'normal'
        i += 1
        continue
    elif state == 'block_comment':
        if s.startswith('*/', i):
            out_chars.append('*/')
            i += 2
            state = 'normal'
        else:
            out_chars.append(c)
            i += 1
        continue
    elif state == 'string':
        if c == '\\' and i + 1 < length:
            out_chars.append(c)
            out_chars.append(s[i+1])
            i += 2
            continue
        elif c == '"':
            out_chars.append(c)
            i += 1
            state = 'normal'
            continue
        else:
            out_chars.append(c)
            i += 1
            continue
    elif state == 'char':
        if c == '\\' and i + 1 < length:
            out_chars.append(c)
            out_chars.append(s[i+1])
            i += 2
            continue
        elif c == "'":
            out_chars.append(c)
            i += 1
            state = 'normal'
            continue
        else:
            out_chars.append(c)
            i += 1
            continue

new_s = ''.join(out_chars)

# Trim trailing whitespace on lines created by removed comments
new_lines = [ln.rstrip() for ln in new_s.splitlines()]
new_s = '\n'.join(new_lines) + '\n'

print(f"Writing cleaned file: {p}")
p.write_text(new_s, encoding='utf-8')
print("Done.")
