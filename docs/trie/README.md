# Read each token prefix once

[Reading page](https://masonmcbride.github.io/outlines-core/) · [Notebook](Prefix_Sharing.ipynb)

The opt-in Rust implementation is in `src/index/trie.rs`; tests are in
`tests/trie_equivalence.rs`. The original constructor remains the default.

## Check

From the repository root, restore the pinned lockfile, then run:

```sh
python3 docs/trie/prepare_vocab.py --lock-only
cargo +1.90.0 test --locked --no-default-features --features trie-experiment --lib --test trie_equivalence
python3 -m unittest docs.trie.test_prepare_vocab
python3 docs/trie/check_artifacts.py
```

## Rebuild the reading artifacts

```sh
uv run --quiet --no-project --python 3.12 \
  --with matplotlib==3.11.2 --with nbformat==5.10.4 \
  --with nbclient==0.10.2 --with markdown==3.11.1 \
  --with ipykernel==6.30.1 python docs/trie/build_notebook.py
```

This reads saved samples; it does not rerun the benchmark. `data/` contains
the three primary datasets, environment, and pinned lockfile as gzip fixtures.
Hashes verify both compressed files and original bytes. For example,
`gzip -dc docs/trie/data/qwen.json.gz` prints all Qwen observations.
The full reports and separate-process repeats remain in the commit identified
by `SOURCE_PROVENANCE.json`.

## Rerun the benchmark

```sh
python3 docs/trie/prepare_vocab.py
cargo +1.90.0 build --release --locked --no-default-features --features trie-experiment --example trie_benchmark
taskset -c 0 target/release/examples/trie_benchmark reruns/vocab/qwen-vocabulary.json reruns/qwen.json 21
```

Use an available CPU if affinity excludes 0. Substitute `gpt2-vocabulary.json`
for GPT-2, or `synthetic` for the synthetic control. The command restores inputs
under ignored paths; saved measurements remain untouched.

The `publish-trie.yml` workflow publishes this folder through GitHub Pages.
These measurements concern index construction, not generation throughput.
