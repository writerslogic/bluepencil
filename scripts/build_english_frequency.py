#!/usr/bin/env python3
"""Rebuild crates/bluepencil-core/data/english_frequency.tsv from the Google Books
Ngram Viewer 1-gram English dataset (version 20120701).

Source: https://books.google.com/ngrams/info -- "Ngram Viewer graphs and data may
be freely used for any purpose." No share-alike, attribution merely appreciated.

Streams each per-letter file, sums match_count across all years per lowercased
word, and writes the top N words as tab-separated (word, per-million frequency),
sorted most frequent first. Never writes the raw decompressed data to disk.

Usage: uv run python scripts/build_english_frequency.py [--top N] [--out PATH]
"""

import argparse
import gzip
import re
import sys
import urllib.request
from collections import Counter

BASE = "http://storage.googleapis.com/books/ngrams/books/googlebooks-eng-all-1gram-20120701-{}.gz"
SHARDS = list("0123456789") + list("abcdefghijklmnopqrstuvwxyz")
WORD_RE = re.compile(r"^[a-z]+(?:'[a-z]+)?$")


def shard_counts(shard: str) -> Counter:
    counts: Counter = Counter()
    url = BASE.format(shard)
    print(f"  {shard}: downloading {url}", file=sys.stderr)
    with urllib.request.urlopen(url) as response, gzip.GzipFile(fileobj=response) as stream:
        for raw_line in stream:
            line = raw_line.decode("utf-8", "ignore")
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 4:
                continue
            ngram, _year, match_count, _volume_count = fields
            word = ngram.lower()
            if not WORD_RE.match(word):
                continue
            counts[word] += int(match_count)
    print(f"  {shard}: {len(counts)} distinct words", file=sys.stderr)
    return counts


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--top", type=int, default=15000)
    parser.add_argument("--out", default="crates/bluepencil-core/data/english_frequency.tsv")
    parser.add_argument("--shards", default=",".join(SHARDS), help="comma-separated shard letters/digits to process")
    args = parser.parse_args()

    total = Counter()
    for shard in args.shards.split(","):
        total.update(shard_counts(shard))

    grand_total = sum(total.values())
    if grand_total == 0:
        raise SystemExit("no words counted -- aborting rather than writing an empty corpus")

    ranked = total.most_common(args.top)
    with open(args.out, "w", encoding="utf-8") as f:
        f.write("# word\tper_million\n")
        f.write(f"# Source: Google Books Ngram Viewer 1-gram English dataset (20120701),\n")
        f.write(f"# https://books.google.com/ngrams/info -- freely usable, no share-alike.\n")
        f.write(f"# Regenerate with scripts/build_english_frequency.py. Top {len(ranked)} words\n")
        f.write(f"# by total occurrence across a corpus of {grand_total} tokens.\n")
        for word, count in ranked:
            per_million = count / grand_total * 1_000_000
            f.write(f"{word}\t{per_million:.4f}\n")

    print(f"wrote {len(ranked)} words to {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
