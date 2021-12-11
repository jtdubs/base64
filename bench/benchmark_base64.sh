#!/bin/sh

set -e

if [ ! -e files/test-empty  ]; then truncate -s 0    files/test-empty;  fi
if [ ! -e files/test-small  ]; then truncate -s 100K files/test-small;  fi
if [ ! -e files/test-medium ]; then truncate -s 10M  files/test-medium; fi
if [ ! -e files/test-large  ]; then truncate -s 1G   files/test-large;  fi

if [ ! -e files/test-empty.b64  ]; then base64 files/test-empty  > files/test-empty.b64;  fi
if [ ! -e files/test-small.b64  ]; then base64 files/test-small  > files/test-small.b64;  fi
if [ ! -e files/test-medium.b64 ]; then base64 files/test-medium > files/test-medium.b64; fi
if [ ! -e files/test-large.b64  ]; then base64 files/test-large  > files/test-large.b64;  fi

hyperfine --warmup 1 -n coreutils 'base64       files/test-empty           > /dev/null' -n rust '../target/release/base64       files/test-empty           > /dev/null' --export-markdown results/b64-empty.md
hyperfine --warmup 1 -n coreutils 'base64       files/test-large           > /dev/null' -n rust '../target/release/base64       files/test-large           > /dev/null' --export-markdown results/b64-file-encode-wrap-large.md
hyperfine --warmup 1 -n coreutils 'base64 -w 0  files/test-large           > /dev/null' -n rust '../target/release/base64 -w 0  files/test-large           > /dev/null' --export-markdown results/b64-file-encode-nowrap-large.md
hyperfine --warmup 1 -n coreutils 'base64 -d    files/test-large.b64       > /dev/null' -n rust '../target/release/base64 -d    files/test-large.b64       > /dev/null' --export-markdown results/b64-file-decode-noignore-large.md
hyperfine --warmup 1 -n coreutils 'base64 -d -i files/test-large.b64       > /dev/null' -n rust '../target/release/base64 -d -i files/test-large.b64       > /dev/null' --export-markdown results/b64-file-decode-ignore-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large     | base64       > /dev/null' -n rust 'cat files/test-large     | ../target/release/base64       > /dev/null' --export-markdown results/b64-stdin-encode-wrap-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large     | base64 -w 0  > /dev/null' -n rust 'cat files/test-large     | ../target/release/base64 -w 0  > /dev/null' --export-markdown results/b64-stdin-encode-nowrap-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large.b64 | base64 -d    > /dev/null' -n rust 'cat files/test-large.b64 | ../target/release/base64 -d    > /dev/null' --export-markdown results/b64-stdin-decode-noignore-large.md
hyperfine --warmup 1 -n coreutils 'cat files/test-large.b64 | base64 -d -i > /dev/null' -n rust 'cat files/test-large.b64 | ../target/release/base64 -d -i > /dev/null' --export-markdown results/b64-stdin-decode-ignore-large.md
