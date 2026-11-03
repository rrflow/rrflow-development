#!/usr/bin/env python3
"""Turn an ann-benchmarks HDF5 file into the pinned files the lab reads.

The lab keeps no dataset in the repository. This writes the train and test vectors as
little-endian 32-bit floats, each named by its own SHA-256, and prints what a manifest
needs. Before writing, it checks the file's published neighbors against an exact search,
so a damaged or misread file is refused here rather than graded against later.

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


def write_pinned(values, directory):
    data = numpy.asarray(values, dtype="<f4").tobytes()
    digest = hashlib.sha256(data).hexdigest()
    (directory / f"{digest}.f32le").write_bytes(data)
    return digest


def exact_neighbors(train, queries, distance, k):
    if distance == "angular":
        train = train / numpy.maximum(numpy.linalg.norm(train, axis=1, keepdims=True), 1e-12)
        queries = queries / numpy.maximum(numpy.linalg.norm(queries, axis=1, keepdims=True), 1e-12)
    found = []
    for start in range(0, len(queries), BATCH):
        batch = queries[start : start + BATCH]
        if distance == "angular":
            scores = batch @ train.T
        elif distance == "euclidean":
            scores = -(
                numpy.sum(batch**2, axis=1, keepdims=True)
                - 2 * batch @ train.T
                + numpy.sum(train**2, axis=1)
            )
        else:
            raise SystemExit(f"distance {distance} has no exact check here")
        found.append(numpy.argsort(-scores, axis=1)[:, :k])
    return numpy.concatenate(found)


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
    count = min(arguments.check_queries, len(test))
    k = min(10, published.shape[1])
    exact = exact_neighbors(train, test[:count], distance, k)
    overlap = numpy.mean(
        [len(set(left) & set(right)) / k for left, right in zip(exact, published[:count, :k])]
    )
    # Ties can reorder equal neighbors, so a small shortfall is expected and a large one is not.
    if overlap < 0.99:
        raise SystemExit(f"the file's neighbors agree with an exact search on only {overlap:.4f}")
    arguments.output.mkdir(parents=True, exist_ok=True)
    json.dump(
        {
            "source_sha256": source_digest,
            "points": len(train),
            "queries": len(test),
            "dimensions": train.shape[1],
            "distance": distance,
            "neighbor_agreement": round(float(overlap), 6),
            "points_sha256": write_pinned(train, arguments.output),
            "queries_sha256": write_pinned(test, arguments.output),
        },
        sys.stdout,
        indent=2,
    )
    print()


if __name__ == "__main__":
    main()
