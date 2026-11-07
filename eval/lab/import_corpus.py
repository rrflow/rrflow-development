#!/usr/bin/env python3
"""Turn an ann-benchmarks HDF5 file into the pinned files the lab reads.

The lab keeps no dataset in the repository. This writes the train and test vectors as
little-endian 32-bit floats, each named by its own SHA-256, and prints what a manifest
needs. Before writing, it scores the file's published neighbors with an exact search, so
a damaged or misread file is refused here. The lab grades with its own exact answers from
these vectors, so the published neighbors decide nothing else. Some files publish neighbors
computed at lower precision, which is why this compares scores and not identities.

Usage: import_corpus.py HDF5_FILE OUTPUT_DIRECTORY [--check-queries N]
Install from import-requirements.txt, which is locked by hash.
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

import h5py
import numpy

BATCH = 64
# Largest gap allowed between the exact and the published score at one rank, as a share of
# the score's scale. A misread or damaged file misses this by orders of magnitude.
MAX_SCORE_GAP = 0.002


def write_pinned(values, directory):
    data = numpy.asarray(values, dtype="<f4").tobytes()
    digest = hashlib.sha256(data).hexdigest()
    (directory / f"{digest}.f32le").write_bytes(data)
    return digest


def scores_against(train, queries, distance):
    if distance == "angular":
        train = train / numpy.maximum(numpy.linalg.norm(train, axis=1, keepdims=True), 1e-12)
        queries = queries / numpy.maximum(numpy.linalg.norm(queries, axis=1, keepdims=True), 1e-12)
        return queries @ train.T
    if distance == "euclidean":
        return -numpy.sqrt(
            numpy.maximum(
                numpy.sum(queries**2, axis=1, keepdims=True)
                - 2 * queries @ train.T
                + numpy.sum(train**2, axis=1),
                0,
            )
        )
    raise SystemExit(f"distance {distance} has no exact check here")


def largest_gap_and_agreement(train, queries, published, distance, k):
    gaps, agreement = [], []
    for start in range(0, len(queries), BATCH):
        batch = queries[start : start + BATCH]
        scores = scores_against(train, batch, distance)
        order = numpy.argsort(-scores, axis=1)[:, :k]
        theirs = published[start : start + len(batch), :k]
        ours_scores = numpy.take_along_axis(scores, order, axis=1)
        their_scores = numpy.take_along_axis(scores, theirs, axis=1)
        scale = numpy.maximum(numpy.abs(ours_scores), 1e-6)
        gaps.append(numpy.max(numpy.abs(ours_scores - their_scores) / scale))
        agreement.extend(len(set(a) & set(b)) / k for a, b in zip(order, theirs))
    return float(max(gaps)), float(numpy.mean(agreement))


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--check-queries", type=int, default=200)
    arguments = parser.parse_args()
    source_digest = hashlib.sha256(arguments.source.read_bytes()).hexdigest()
    with h5py.File(arguments.source, "r") as source:
        train = numpy.asarray(source["train"], dtype=numpy.float32)
        test = numpy.asarray(source["test"], dtype=numpy.float32)
        published = numpy.asarray(source["neighbors"])
        distance = source.attrs["distance"]
    if isinstance(distance, bytes):
        distance = distance.decode()
    if published.max() >= len(train) or published.min() < 0:
        raise SystemExit("the file's published neighbors point outside its vectors")
    count = min(arguments.check_queries, len(test))
    k = min(10, published.shape[1])
    gap, agreement = largest_gap_and_agreement(train, test[:count], published, distance, k)
    if gap > MAX_SCORE_GAP:
        raise SystemExit(f"the file's published neighbors score {gap:.6f} away from an exact search")
    arguments.output.mkdir(parents=True, exist_ok=True)
    json.dump(
        {
            "source_sha256": source_digest,
            "points": len(train),
            "queries": len(test),
            "dimensions": train.shape[1],
            "distance": distance,
            "neighbor_agreement": round(agreement, 6),
            "largest_score_gap": round(gap, 6),
            "points_sha256": write_pinned(train, arguments.output),
            "queries_sha256": write_pinned(test, arguments.output),
        },
        sys.stdout,
        indent=2,
    )
    print()


if __name__ == "__main__":
    main()
