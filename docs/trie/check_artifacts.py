"""Validate saved data, exact source excerpts, and executed notebook outputs."""
import hashlib
import gzip
import json
from pathlib import Path
import statistics

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
p=json.loads((HERE/'SOURCE_PROVENANCE.json').read_text())
for name,digest in p['rust_source_sha256'].items():
    raw=(ROOT/name).read_bytes() if (ROOT/name).exists() else gzip.decompress((HERE/'data/Cargo.lock.gz').read_bytes())
    assert hashlib.sha256(raw).hexdigest()==digest,name
for name,digest in p['measurements_sha256'].items():
    assert hashlib.sha256((HERE/name).read_bytes()).hexdigest()==digest,name
for name,digest in p['compressed_tokenizers_sha256'].items():
    assert hashlib.sha256((HERE/name).read_bytes()).hexdigest()==digest,name
assert hashlib.sha256((HERE/'data/Cargo.lock.gz').read_bytes()).hexdigest()==p['compressed_lockfile_sha256']
for name,digest in p['uncompressed_measurements_sha256'].items():
    assert hashlib.sha256(gzip.decompress((HERE/name).read_bytes())).hexdigest()==digest,name
for name in ['gpt2','qwen','synthetic']:
    d=json.loads(gzip.decompress((HERE/f'data/{name}.json.gz').read_bytes()))
    assert len(d['cases'])==6
    for c in d['cases']:
        assert c['exact_index_equality']
        for v in ['reference','cold_trie','warm_trie']:
            assert len(c[v]['samples_ms'])==21
            assert statistics.median(c[v]['samples_ms'])==c[v]['median_ms']
    if name!='synthetic':
        raw=HERE/'vocab'/f"{name.replace('-repeat','')}-tokenizer.json.gz"
        assert hashlib.sha256(gzip.decompress(raw.read_bytes())).hexdigest()==d['vocabulary']['raw_sha256']
notebook=json.loads((HERE/'Prefix_Sharing.ipynb').read_text())
codes=[c for c in notebook['cells'] if c['cell_type']=='code']
assert [c['execution_count'] for c in codes]==[1,2,3]
assert not any(o['output_type']=='error' for c in codes for o in c['outputs'])
exact=next(''.join(c['source']) for c in notebook['cells'] if 'exact Rust source' in ''.join(c['source']))
excerpt=exact.split('```rust\n')[1].split('\n```')[0]
import textwrap
source=(ROOT/'src/index/trie.rs').read_text()
start=source.index('            stack.push((0, current_state));')
end=source.index('\n            for (rank, candidate)',start)
assert excerpt==textwrap.dedent(source[start:end]).strip()
base=p['upstream_commit']
import subprocess
original=subprocess.check_output(['git','show',base+':src/index.rs'],cwd=ROOT,text=True)
addition='#[cfg(feature = "trie-experiment")]\nmod trie;\n#[cfg(feature = "trie-experiment")]\npub use trie::{TrieWork, VocabularyTrie};\n\n'
assert (ROOT/'src/index.rs').read_text().replace(addition,'',1)==original
print('Three primary datasets, source hashes, unchanged baseline, exact excerpt, and three executed cells passed')
