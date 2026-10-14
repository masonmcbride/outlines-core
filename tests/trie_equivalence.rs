#![cfg(feature = "trie-experiment")]

use outlines_core::index::VocabularyTrie;
use outlines_core::prelude::*;
use rustc_hash::FxHashMap;

fn compare(regex: &str, vocabulary: &Vocabulary) {
    let baseline = Index::new(regex, vocabulary);
    let trie = VocabularyTrie::new(vocabulary);
    let candidate = Index::new_with_prepared_trie(regex, &trie);
    match (&baseline, &candidate) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "regex {regex:?}"),
        (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string(), "regex {regex:?}"),
        _ => panic!("Construction status mismatch for {regex:?}: {baseline:?} / {candidate:?}"),
    }
    let cold = Index::new_with_trie(regex, vocabulary);
    match (&candidate, &cold) {
        (Ok(a), Ok(b)) => assert_eq!(a, b),
        (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string()),
        _ => panic!("Prepared/cold construction mismatch"),
    }
}

fn vocabulary(tokens: &[&[u8]]) -> Vocabulary {
    let mut vocabulary = Vocabulary::new(100_000);
    for (id, token) in tokens.iter().enumerate() {
        vocabulary.try_insert(token.to_vec(), id as u32).unwrap();
    }
    vocabulary
}

#[test]
fn edge_cases_preserve_exact_indexes_and_errors() {
    let vocabularies = [
        vocabulary(&[]),
        vocabulary(&[b"", b"a", b"b", b"ab", b"aba", b"x"]),
        vocabulary(&[b"0", b"0 ", b"1"]),
        vocabulary(&[
            b"\xc3",
            b"\xa9",
            b"\xc3\xa9",
            b" ",
            b"\xf0",
            b"\x9f",
            b"\x98",
            b"\x87",
        ]),
        vocabulary(&[
            b"a", b"ab", b"abc", b"abcd", b"abcde", b"b", b"c", b"d", b"e",
        ]),
    ];
    for vocabulary in vocabularies {
        for regex in [
            "",
            "a",
            "ab|abc",
            "a*",
            "[ab]{0,3}",
            "(ac|[^a])+",
            "0 1",
            "é|😇",
            "[",
            "^a",
            "a$",
            "(?i)abc",
            r"\bword\b",
            r"(?s:.)*",
        ] {
            compare(regex, &vocabulary);
        }
    }
}

#[test]
fn duplicate_ids_empty_id_lists_and_prefixes_preserve_reference_order() {
    // The public vocabulary API permits IDs shared by different byte strings,
    // duplicate IDs on one string, and entries with no IDs.
    let tokens = FxHashMap::from_iter([
        (b"".to_vec(), vec![7]),
        (b"a".to_vec(), vec![1, 1, 2]),
        (b"ab".to_vec(), vec![1, 3]),
        (b"b".to_vec(), vec![3]),
        (b"x".to_vec(), Vec::new()),
    ]);
    let vocabulary = Vocabulary::try_from((100, tokens)).unwrap();
    for regex in ["a|ab", "[ab]*", "a{1,3}", "x", "xy", ""] {
        compare(regex, &vocabulary);
    }
}

#[test]
fn deterministic_generated_vocabularies() {
    let mut seed = 0xdec0de_u64;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) as usize
    };
    let alphabet = [b'a', b'b', b'c', b'0', b'1', b' ', b'\n', 0xc3, 0xa9];
    let patterns = [
        "[ab]{1,4}",
        "a(b|c)*",
        "0|[1-9][0-9]*",
        "(ab|ba|c)+",
        "é|aé",
        r"[a-c\s]*",
        "",
        "[^a]{1,3}",
        "a{2}|b{3}",
        "[ab]*c",
        "[",
        r"(?s:.)+",
    ];
    for _ in 0..250 {
        let mut vocabulary = Vocabulary::new(10_000);
        // Some vocabularies are complete; others intentionally cannot finish.
        for id in 0..(5 + next() % 25) {
            let length = next() % 7;
            let bytes: Vec<_> = (0..length)
                .map(|_| alphabet[next() % alphabet.len()])
                .collect();
            vocabulary.try_insert(bytes, id as u32).unwrap();
        }
        for pattern in patterns {
            compare(pattern, &vocabulary);
        }
    }
}

fn check_finite_language(regex: &str, words: &[&[u8]], tokens: &[&[u8]]) {
    let vocabulary = vocabulary(tokens);
    let reference = Index::new(regex, &vocabulary).unwrap();
    let candidate = Index::new_with_trie(regex, &vocabulary).unwrap();
    assert_eq!(reference, candidate);
    let mut pending = vec![(candidate.initial_state(), Vec::<u8>::new())];
    while let Some((state, prefix)) = pending.pop() {
        let actual: std::collections::BTreeSet<_> = candidate
            .allowed_tokens(&state)
            .unwrap_or_default()
            .into_iter()
            .collect();
        let mut expected = std::collections::BTreeSet::new();
        if words.contains(&prefix.as_slice()) {
            expected.insert(vocabulary.eos_token_id());
        }
        for (id, token) in tokens.iter().enumerate() {
            let mut extended = prefix.clone();
            extended.extend_from_slice(token);
            if words.iter().any(|word| word.starts_with(&extended)) {
                expected.insert(id as u32);
                let next = candidate.next_state(&state, &(id as u32)).unwrap_or_else(|| panic!("Valid prefix rejected: regex {regex:?}, prefix {prefix:?}, token {token:?}, state {state}, actual {actual:?}"));
                if !token.is_empty() {
                    pending.push((next, extended));
                }
            }
        }
        assert_eq!(actual, expected, "Allowed token mismatch at {prefix:?}");
        assert_eq!(
            candidate.is_final_state(&state),
            words.contains(&prefix.as_slice())
        );
    }
}

#[test]
fn independent_finite_language_oracle() {
    // Finite languages without prefix-overlapping alternatives. Preserve the
    // reference's leftmost-first regex semantics rather than changing them.
    check_finite_language(
        "ab|ba|bb",
        &[b"ab", b"ba", b"bb"],
        &[b"a", b"b", b"ab", b"aba", b"bb", b"ba", b"x", b"aaaa"],
    );
    check_finite_language(
        "(?:a|bb|aba)$",
        &[b"a", b"bb", b"aba"],
        &[b"a", b"b", b"ab", b"aba", b"bb", b"ba", b"x", b"aaaa"],
    );
    check_finite_language(
        "a{0,2}",
        &[b"", b"a", b"aa"],
        &[b"a", b"aa", b"b", b"bb", b"ba", b"x"],
    );
    check_finite_language(
        "café|😇",
        &["café".as_bytes(), "😇".as_bytes()],
        &[
            b"c",
            b"a",
            b"f",
            b"\xc3",
            b"\xa9",
            "é".as_bytes(),
            b"caf",
            b"\xf0",
            b"\x9f",
            b"\x98",
            b"\x87",
            "😇".as_bytes(),
            b"x",
        ],
    );
}

#[test]
fn reusable_trie_and_nonrecursive_long_token() {
    let mut vocabulary = vocabulary(&[b"a", b"b", b"ab"]);
    vocabulary.try_insert(vec![b'a'; 20_000], 20).unwrap();
    let trie = VocabularyTrie::new(&vocabulary);
    assert!(trie.node_count() >= 20_001);
    for regex in ["a*", "ab", "b+", "[ab]{1,3}"] {
        assert_eq!(
            Index::new(regex, &vocabulary).unwrap(),
            Index::new_with_prepared_trie(regex, &trie).unwrap()
        );
    }
}

#[test]
fn preserve_reference_prefix_alternation_and_assertion_behavior() {
    let vocabulary = vocabulary(&[b"a", b"b", b"ab", b"aba", b"bb"]);
    let short_first = Index::new("a|bb|aba", &vocabulary).unwrap();
    let allowed = short_first
        .allowed_tokens(&short_first.initial_state())
        .unwrap();
    assert!(!allowed.contains(&2));
    assert!(!allowed.contains(&3));
    compare("a|bb|aba", &vocabulary);

    let long_first = Index::new("aba|bb|a", &vocabulary).unwrap();
    let allowed = long_first
        .allowed_tokens(&long_first.initial_state())
        .unwrap();
    assert!(allowed.contains(&2));
    assert!(allowed.contains(&3));
    compare("aba|bb|a", &vocabulary);
    for regex in [
        "^(?:a|bb|aba)$",
        "(?:a|bb|aba)$",
        "^(?:a?|bb)$",
        "(?:a?|bb)$",
    ] {
        let result = Index::new(regex, &vocabulary);
        if regex == "(?:a|bb|aba)$" {
            assert!(result.is_ok());
        } else {
            assert!(matches!(
                result,
                Err(outlines_core::Error::DfaHasNoStartState)
            ));
        }
        compare(regex, &vocabulary);
    }
}
