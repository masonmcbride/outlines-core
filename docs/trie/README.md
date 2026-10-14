# Read each token prefix once

[Read the illustrated explanation](index.html) or open [the executed notebook](Prefix_Sharing.ipynb).
The implementation is in `src/index/trie.rs`, enabled by `trie-experiment`.
The original constructor remains the default. This is an experimental Rust
index-construction optimization.

## Run the checks

From the repository root:

```sh
cargo +1.90.0 test --locked --no-default-features --features trie-experiment --lib --test trie_equivalence
python3 -m unittest docs.trie.test_prepare_vocab
cargo +1.90.0 fmt --check
cargo +1.90.0 clippy --locked --no-default-features --features trie-experiment --lib --test trie_equivalence --example trie_benchmark -- -D warnings
```

## Rebuild the notebook and HTML

This reads saved measurements; no model, GPU, or benchmark rerun is required.

```sh
uv run --quiet --no-project --python 3.12 \
  --with matplotlib==3.11.2 --with nbformat==5.10.4 \
  --with nbclient==0.10.2 --with nbconvert==7.16.6 \
  --with ipykernel==6.30.1 python docs/trie/build_notebook.py
python3 docs/trie/check_artifacts.py
```

## Rerun timings

Pinned tokenizer inputs are compressed in `vocab/`; derived vocabularies are rebuilt on demand.
Their origins, revisions, and hashes are recorded in `data/` and `SOURCE_PROVENANCE.json`.
`prepare_vocab.py` implements the inspected ByteLevel inverse mapping.
To collect new timings without overwriting the recorded results:

```sh
cargo +1.90.0 build --release --locked --no-default-features --features trie-experiment --example trie_benchmark
python3 docs/trie/prepare_vocab.py
taskset -c 0 target/release/examples/trie_benchmark reruns/vocab/qwen-vocabulary.json reruns/qwen.json 21
```

Use an available CPU instead of 0 if your affinity excludes it. GPT-2 uses
`gpt2-vocabulary.json`; pass `synthetic` instead of a path for the synthetic control.
Every variant reconstructs the DFA and index; only the prepared variant reuses
the vocabulary trie. These are constructor timings, not generation throughput.

## Application link

Publish the `docs/trie/` folder with the included `publish-trie.yml` Pages workflow.
The reading URL will be `https://masonmcbride.github.io/outlines-core/` once deployed.
Source links point to branch `trie-prefix-sharing`. No upstream acceptance is claimed.
