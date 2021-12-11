#!/bin/sh

set -e

if [ ! -e files/test-empty  ]; then truncate -s 0    files/test-empty;  fi
if [ ! -e files/test-small  ]; then truncate -s 100K files/test-small;  fi
if [ ! -e files/test-medium ]; then truncate -s 10M  files/test-medium; fi
if [ ! -e files/test-large  ]; then truncate -s 1G   files/test-large;  fi

if [ ! -e files/test-empty.b32  ]; then base32 files/test-empty  > files/test-empty.b32;  fi
if [ ! -e files/test-small.b32  ]; then base32 files/test-small  > files/test-small.b32;  fi
if [ ! -e files/test-medium.b32 ]; then base32 files/test-medium > files/test-medium.b32; fi
if [ ! -e files/test-large.b32  ]; then base32 files/test-large  > files/test-large.b32;  fi

echo "empty..."
hyperfine --warmup 1 -n coreutils 'base32       files/test-empty           > /dev/null' -n rust '../target/release/base32       files/test-empty           > /dev/null' --export-markdown results/b32-empty.md
hyperfine --warmup 1 -n coreutils 'base32       files/test-large           > /dev/null' -n rust '../target/release/base32       files/test-large           > /dev/null' --export-markdown results/b32-file-encode-wrap-large.md
hyperfine --warmup 1 -n coreutils 'base32 -w 0  files/test-large           > /dev/null' -n rust '../target/release/base32 -w 0  files/test-large           > /dev/null' --export-markdown results/b32-file-encode-nowrap-large.md
hyperfine --warmup 1 -n coreutils 'base32 -d    files/test-large.b32       > /dev/null' -n rust '../target/release/base32 -d    files/test-large.b32       > /dev/null' --export-markdown results/b32-file-decode-noignore-large.md
hyperfine --warmup 1 -n coreutils 'base32 -d -i files/test-large.b32       > /dev/null' -n rust '../target/release/base32 -d -i files/test-large.b32       > /dev/null' --export-markdown results/b32-file-decode-ignore-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large     | base32       > /dev/null' -n rust 'cat files/test-large     | ../target/release/base32       > /dev/null' --export-markdown results/b32-stdin-encode-wrap-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large     | base32 -w 0  > /dev/null' -n rust 'cat files/test-large     | ../target/release/base32 -w 0  > /dev/null' --export-markdown results/b32-stdin-encode-nowrap-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large.b32 | base32 -d    > /dev/null' -n rust 'cat files/test-large.b32 | ../target/release/base32 -d    > /dev/null' --export-markdown results/b32-stdin-decode-noignore-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large.b32 | base32 -d -i > /dev/null' -n rust 'cat files/test-large.b32 | ../target/release/base32 -d -i > /dev/null' --export-markdown results/b32-stdin-decode-ignore-large.md
