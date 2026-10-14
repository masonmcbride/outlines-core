import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest

from docs.trie.prepare_vocab import byte_decoder, normalize


class ByteVocabularyTests(unittest.TestCase):
    def test_mapping_is_bijective_and_preserves_partial_utf8(self):
        mapping = byte_decoder()
        self.assertEqual(len(mapping), 256)
        self.assertEqual(set(mapping.values()), set(range(256)))
        self.assertEqual(bytes(mapping[c] for c in "Ġhello"), b" hello")
        self.assertEqual(bytes(mapping[c] for c in "Ã©"), "é".encode())
        self.assertEqual(bytes(mapping[c] for c in "Ã"), b"\xc3")

    def test_fixture_normalization_excludes_specials_and_records_hash(self):
        inverse = {byte: char for char, byte in byte_decoder().items()}
        source = {
            "decoder": {"type": "ByteLevel"},
            "added_tokens": [{"id": 1000, "special": True}],
            "model": {"vocab": {**{inverse[b]: b for b in range(256)}, "<special>": 1000, "Ġhello": 256}},
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "test-tokenizer.json"
            path.write_text(json.dumps(source))
            with contextlib.redirect_stdout(io.StringIO()):
                normalize(path, 1000, "test fixture")
            result = json.loads(path.with_name("test-vocabulary.json").read_text())
            self.assertEqual(result["metadata"]["regular_token_ids"], 257)
            self.assertEqual(result["metadata"]["raw_sha256"], hashlib.sha256(path.read_bytes()).hexdigest())
            self.assertEqual(result["eos_token_id"], 1000)
            self.assertTrue(any(t["bytes"] == list(b" hello") and t["ids"] == [256] for t in result["tokens"]))
            self.assertFalse(any(1000 in t["ids"] for t in result["tokens"]))


if __name__ == "__main__":
    unittest.main()
