"""Normalize pinned ByteLevel tokenizers using the core's CHAR_MAP mapping."""
import hashlib
import json
from pathlib import Path


def byte_decoder():
    mapping = {}
    extra = 256
    for byte in range(256):
        if 33 <= byte <= 126 or 161 <= byte <= 172 or 174 <= byte <= 255:
            mapping[chr(byte)] = byte
        else:
            mapping[chr(extra)] = byte
            extra += 1
    return mapping


def normalize(path, eos_token_id, source):
    raw = path.read_bytes()
    tokenizer = json.loads(raw)
    assert tokenizer["decoder"]["type"] == "ByteLevel"
    decode = byte_decoder()
    special_ids = {t["id"] for t in tokenizer["added_tokens"] if t["special"]}
    grouped = {}
    for text, token_id in tokenizer["model"]["vocab"].items():
        if token_id in special_ids:
            continue
        token_bytes = bytes(decode[char] for char in text)
        grouped.setdefault(token_bytes, []).append(token_id)
    # Both selected ByteLevel vocabularies include all standalone bytes.
    assert all(bytes([byte]) in grouped for byte in range(256))
    payload = {
        "eos_token_id": eos_token_id,
        "tokens": [{"bytes": list(token), "ids": sorted(ids)} for token, ids in sorted(grouped.items())],
        "metadata": {
            "source": source,
            "raw_sha256": hashlib.sha256(raw).hexdigest(),
            "normalization": "ByteLevel inverse CHAR_MAP from src/vocabulary/processor.rs; special tokens excluded",
            "regular_token_ids": sum(map(len, grouped.values())),
        },
    }
    output = path.with_name(path.stem.replace("-tokenizer", "-vocabulary") + ".json")
    output.write_text(json.dumps(payload, separators=(",", ":")) + "\n")
    print(output.name, payload["metadata"]["regular_token_ids"], "regular token IDs")


if __name__ == "__main__":
    import gzip
    root = Path(__file__).resolve().parents[2]
    folder = root / "reruns/vocab"
    folder.mkdir(parents=True, exist_ok=True)
    for name, eos in [("gpt2", 50256), ("qwen", 151645)]:
        here = Path(__file__).resolve().parent
        data = json.loads((here / "data" / (name + ".json")).read_text())
        compressed = here / "vocab" / (name + "-tokenizer.json.gz")
        raw = gzip.decompress(compressed.read_bytes())
        assert hashlib.sha256(raw).hexdigest() == data["vocabulary"]["raw_sha256"]
        path = folder / (name + "-tokenizer.json")
        path.write_bytes(raw)
        normalize(path, eos, data["vocabulary"]["source"])
