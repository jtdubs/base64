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

# cargo build --release
# hyperfine --warmup 0 -n rust '../target/release/base64 -d    files/test-large.b64  > /dev/null'

# echo "empty..."
# hyperfine --warmup 1 -n coreutils 'base64       files/test-empty      > /dev/null' -n rust '../target/release/base64       files/test-empty      > /dev/null' --export-markdown results/empty.md

# echo "Benchmarking wrapping encoding..."
# hyperfine --warmup 1 -n coreutils 'base64       files/test-small      > /dev/null' -n rust '../target/release/base64       files/test-small      > /dev/null' --export-markdown results/file-encode-wrap-small.md
# hyperfine --warmup 1 -n coreutils 'base64       files/test-medium     > /dev/null' -n rust '../target/release/base64       files/test-medium     > /dev/null' --export-markdown results/file-encode-wrap-medium.md
# hyperfine --warmup 1 -n coreutils 'base64       files/test-large      > /dev/null' -n rust '../target/release/base64       files/test-large      > /dev/null' --export-markdown results/file-encode-wrap-large.md

# echo "Benchmarking non-wrapping encoding..."
# hyperfine --warmup 1 -n coreutils 'base64 -w 0  files/test-empty      > /dev/null' -n rust '../target/release/base64 -w 0  files/test-empty      > /dev/null' --export-markdown results/file-encode-nowrap-empty.md
# hyperfine --warmup 1 -n coreutils 'base64 -w 0  files/test-small      > /dev/null' -n rust '../target/release/base64 -w 0  files/test-small      > /dev/null' --export-markdown results/file-encode-nowrap-small.md
# hyperfine --warmup 1 -n coreutils 'base64 -w 0  files/test-medium     > /dev/null' -n rust '../target/release/base64 -w 0  files/test-medium     > /dev/null' --export-markdown results/file-encode-nowrap-medium.md
# hyperfine --warmup 1 -n coreutils 'base64 -w 0  files/test-large      > /dev/null' -n rust '../target/release/base64 -w 0  files/test-large      > /dev/null' --export-markdown results/file-encode-nowrap-large.md

# echo "Benchmarking decoding..."
# hyperfine --warmup 1 -n coreutils 'base64 -d    files/test-empty.b64  > /dev/null' -n rust '../target/release/base64 -d    files/test-empty.b64  > /dev/null' --export-markdown results/file-decode-noignore-empty.md
# hyperfine --warmup 1 -n coreutils 'base64 -d    files/test-small.b64  > /dev/null' -n rust '../target/release/base64 -d    files/test-small.b64  > /dev/null' --export-markdown results/file-decode-noignore-small.md
# hyperfine --warmup 1 -n coreutils 'base64 -d    files/test-medium.b64 > /dev/null' -n rust '../target/release/base64 -d    files/test-medium.b64 > /dev/null' --export-markdown results/file-decode-noignore-medium.md
hyperfine --warmup 1 -n coreutils 'base64 -d    files/test-large.b64  > /dev/null' -n rust '../target/release/base64 -d    files/test-large.b64  > /dev/null' --export-markdown results/file-decode-noignore-large.md

# echo "Benchmarking decoding with ignore_garbage..."
# hyperfine --warmup 1 -n coreutils 'base64 -d -i files/test-empty.b64  > /dev/null' -n rust '../target/release/base64 -d -i files/test-empty.b64  > /dev/null' --export-markdown results/file-decode-ignore-empty.md
# hyperfine --warmup 1 -n coreutils 'base64 -d -i files/test-small.b64  > /dev/null' -n rust '../target/release/base64 -d -i files/test-smal.b64l  > /dev/null' --export-markdown results/file-decode-ignore-small.md
# hyperfine --warmup 1 -n coreutils 'base64 -d -i files/test-medium.b64 > /dev/null' -n rust '../target/release/base64 -d -i files/test-medium.b64 > /dev/null' --export-markdown results/file-decode-ignore-medium.md
# hyperfine --warmup 1 -n coreutils 'base64 -d -i files/test-large.b64  > /dev/null' -n rust '../target/release/base64 -d -i files/test-large.b64  > /dev/null' --export-markdown results/file-decode-ignore-large.md


# echo "Benchmarking wrapping encoding..."
# hyperfine --warmup 1 -n coreutils 'cat files/test-empty      | base64       > /dev/null' -n rust 'cat files/test-empty      | ../target/release/base64       > /dev/null' --export-markdown results/stdin-encode-wrap-empty.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-small      | base64       > /dev/null' -n rust 'cat files/test-small      | ../target/release/base64       > /dev/null' --export-markdown results/stdin-encode-wrap-small.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-medium     | base64       > /dev/null' -n rust 'cat files/test-medium     | ../target/release/base64       > /dev/null' --export-markdown results/stdin-encode-wrap-medium.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-large      | base64       > /dev/null' -n rust 'cat files/test-large      | ../target/release/base64       > /dev/null' --export-markdown results/stdin-encode-wrap-large.md

# echo "Benchmarking non-wrapping encoding..."
# hyperfine --warmup 1 -n coreutils 'cat files/test-empty      | base64 -w 0  > /dev/null' -n rust 'cat files/test-empty      | ../target/release/base64 -w 0  > /dev/null' --export-markdown results/stdin-encode-nowrap-empty.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-small      | base64 -w 0  > /dev/null' -n rust 'cat files/test-small      | ../target/release/base64 -w 0  > /dev/null' --export-markdown results/stdin-encode-nowrap-small.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-medium     | base64 -w 0  > /dev/null' -n rust 'cat files/test-medium     | ../target/release/base64 -w 0  > /dev/null' --export-markdown results/stdin-encode-nowrap-medium.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-large      | base64 -w 0  > /dev/null' -n rust 'cat files/test-large      | ../target/release/base64 -w 0  > /dev/null' --export-markdown results/stdin-encode-nowrap-large.md

# echo "Benchmarking decoding..."
# hyperfine --warmup 1 -n coreutils 'cat files/test-empty.b64  | base64 -d    > /dev/null' -n rust 'cat files/test-empty.b64  | ../target/release/base64 -d    > /dev/null' --export-markdown results/stdin-decode-noignore-empty.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-small.b64  | base64 -d    > /dev/null' -n rust 'cat files/test-small.b64  | ../target/release/base64 -d    > /dev/null' --export-markdown results/stdin-decode-noignore-small.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-medium.b64 | base64 -d    > /dev/null' -n rust 'cat files/test-medium.b64 | ../target/release/base64 -d    > /dev/null' --export-markdown results/stdin-decode-noignore-medium.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-large.b64  | base64 -d    > /dev/null' -n rust 'cat files/test-large.b64  | ../target/release/base64 -d    > /dev/null' --export-markdown results/stdin-decode-noignore-large.md

# echo "Benchmarking decoding with ignore_garbage..."
# hyperfine --warmup 1 -n coreutils 'cat files/test-empty.b64  | base64 -d -i > /dev/null' -n rust 'cat files/test-empty.b64  | ../target/release/base64 -d -i > /dev/null' --export-markdown results/stdin-decode-ignore-empty.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-small.b64  | base64 -d -i > /dev/null' -n rust 'cat files/test-smal.b64l  | ../target/release/base64 -d -i > /dev/null' --export-markdown results/stdin-decode-ignore-small.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-medium.b64 | base64 -d -i > /dev/null' -n rust 'cat files/test-medium.b64 | ../target/release/base64 -d -i > /dev/null' --export-markdown results/stdin-decode-ignore-medium.md
# hyperfine --warmup 1 -n coreutils 'cat files/test-large.b64  | base64 -d -i > /dev/null' -n rust 'cat files/test-large.b64  | ../target/release/base64 -d -i > /dev/null' --export-markdown results/stdin-decode-ignore-large.md
