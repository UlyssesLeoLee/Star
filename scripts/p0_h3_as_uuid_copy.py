#!/usr/bin/env python3
import re
from pathlib import Path
for d in ['domain-audit', 'domain-automation', 'domain-scm', 'domain-workspace']:
    for f in [Path('crates')/d/'src'/'macros.rs', Path('crates')/d/'src'/'lib.rs']:
        if f.exists():
            text = f.read_text(encoding='utf-8')
            new = re.sub(
                r'pub fn as_uuid\(&self\) -> &uuid::Uuid \{\s*&self\.0 \}',
                'pub fn as_uuid(&self) -> uuid::Uuid { self.0 }',
                text
            )
            if new != text:
                f.write_text(new, encoding='utf-8')
                print(f'{d}/{f.name} patched')
            else:
                if 'pub fn as_uuid' in text:
                    print(f'{d}/{f.name} pattern not found, current line:')
                    for line in text.split('\n'):
                        if 'as_uuid' in line:
                            print(f'  {line.strip()}')
