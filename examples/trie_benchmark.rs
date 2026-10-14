//! Reproducible constructor benchmark; no neural model or network calls.
use outlines_core::index::VocabularyTrie;
use outlines_core::prelude::*;
use regex_automata::dfa::{dense::DFA, Automaton};
use regex_automata::util::primitives::StateID;
use serde_json::{json, Value};
use std::{hint::black_box, time::Instant};

fn load_vocabulary(path: &str) -> (Vocabulary, Value) {
    if path == "synthetic" {
        let mut vocabulary = Vocabulary::new(1_000_000);
        for byte in 0..=255u8 {
            vocabulary.try_insert(vec![byte], byte as u32).unwrap();
        }
        for id in 0..20_000u32 {
            vocabulary
                .try_insert(
                    format!("transaction_{id:05}_completed").into_bytes(),
                    id + 256,
                )
                .unwrap();
        }
        return (
            vocabulary,
            json!({"source": "generated 20,000 shared-prefix tokens plus 256 single-byte tokens", "kind": "synthetic"}),
        );
    }
    let data: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let mut vocabulary = Vocabulary::new(data["eos_token_id"].as_u64().unwrap() as u32);
    for token in data["tokens"].as_array().unwrap() {
        let bytes: Vec<u8> = token["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_u64().unwrap() as u8)
            .collect();
        for id in token["ids"].as_array().unwrap() {
            vocabulary
                .try_insert(bytes.clone(), id.as_u64().unwrap() as u32)
                .unwrap();
        }
    }
    (vocabulary, data["metadata"].clone())
}

fn naive_byte_work(regex: &str, vocabulary: &Vocabulary, index: &Index) -> usize {
    let dfa = DFA::new(regex).unwrap();
    let mut count = 0;
    for &state in index.transitions().keys() {
        for (bytes, ids) in vocabulary.tokens() {
            if ids.contains(&vocabulary.eos_token_id()) {
                continue;
            }
            let mut state = StateID::new(state as usize).unwrap();
            for &byte in bytes {
                state = dfa.next_state(state, byte);
                count += 1;
                if dfa.is_dead_state(state) || dfa.is_quit_state(state) {
                    break;
                }
            }
        }
    }
    count
}

fn summary(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    json!({"median_ms": sorted[sorted.len()/2], "min_ms": sorted[0], "max_ms": sorted[sorted.len()-1], "samples_ms": samples})
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 3,
        "usage: trie_benchmark VOCABULARY_JSON|synthetic OUTPUT_JSON [SAMPLES]"
    );
    let samples: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(15);
    assert!(samples > 0);
    let (vocabulary, metadata) = load_vocabulary(&args[1]);
    let trie = VocabularyTrie::new(&vocabulary);
    let total_token_bytes: usize = vocabulary.tokens().keys().map(Vec::len).sum();
    let schema = json!({"type":"object","properties":{
        "name":{"type":"string","maxLength":16},
        "age":{"type":"integer"},
        "role":{"enum":["admin","user","viewer"]}
    },"required":["name","age","role"],"additionalProperties":false});
    let schema_regex = json_schema::regex_from_value(&schema, Some(r"[ \n\r\t]*"), None).unwrap();
    let cases = [
        ("three_digits", r"[0-9]{3}".to_string()),
        ("integer", r"0|[1-9][0-9]*".to_string()),
        ("identifier", r"[A-Za-z_][A-Za-z0-9_]{0,31}".to_string()),
        ("email", r"[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?".to_string()),
        ("json_record", schema_regex),
        ("free_text", r"(?s:.)*".to_string()),
    ];
    let mut trie_samples = Vec::new();
    for _ in 0..samples {
        let started = Instant::now();
        let prepared = black_box(VocabularyTrie::new(black_box(&vocabulary)));
        trie_samples.push(started.elapsed().as_secs_f64() * 1000.0);
        black_box(&prepared);
    }
    let mut output = json!({
        "format_version":1, "vocabulary":metadata,
        "regular_token_ids": vocabulary.len()-1, "unique_byte_strings":vocabulary.tokens().len(),
        "total_unique_token_bytes":total_token_bytes, "trie_nodes":trie.node_count(),
        "trie_payload_estimate_bytes":trie.estimated_heap_bytes(), "trie_preparation":summary(&trie_samples),
        "method":"release; one warm-up per variant; rotating order; constructor timing excludes returned Index destruction; cold trie includes temporary trie destruction; input normalization and counters excluded; warm reuses only vocabulary trie, not DFA or Index",
        "cases":[]
    });
    for (name, regex) in cases {
        let baseline = Index::new(&regex, &vocabulary).unwrap();
        let candidate = Index::new_with_prepared_trie(&regex, &trie).unwrap();
        assert_eq!(baseline, candidate, "Prepared trie differed for {name}");
        assert_eq!(
            baseline,
            Index::new_with_trie(&regex, &vocabulary).unwrap(),
            "Cold trie differed for {name}"
        );
        let naive_bytes = naive_byte_work(&regex, &vocabulary, &baseline);
        let (counted_index, work) = trie.profile_build(&regex).unwrap();
        assert_eq!(baseline, counted_index);
        let transition_count: usize = baseline.transitions().values().map(|m| m.len()).sum();
        drop(baseline);
        drop(candidate);
        drop(counted_index);
        let mut times = [Vec::new(), Vec::new(), Vec::new()];
        // Warm up each variant once. Equality checks above also warm allocations.
        for variant in 0..3 {
            black_box(match variant {
                0 => Index::new(&regex, &vocabulary).unwrap(),
                1 => Index::new_with_trie(&regex, &vocabulary).unwrap(),
                _ => Index::new_with_prepared_trie(&regex, &trie).unwrap(),
            });
        }
        for round in 0..samples {
            for offset in 0..3 {
                let variant = (round + offset) % 3;
                let started = Instant::now();
                let index = black_box(match variant {
                    0 => Index::new(black_box(&regex), black_box(&vocabulary)).unwrap(),
                    1 => Index::new_with_trie(black_box(&regex), black_box(&vocabulary)).unwrap(),
                    _ => {
                        Index::new_with_prepared_trie(black_box(&regex), black_box(&trie)).unwrap()
                    }
                });
                times[variant].push(started.elapsed().as_secs_f64() * 1000.0);
                black_box(&index);
            }
        }
        let reference = summary(&times[0]);
        let cold = summary(&times[1]);
        let warm = summary(&times[2]);
        let warm_speedup =
            reference["median_ms"].as_f64().unwrap() / warm["median_ms"].as_f64().unwrap();
        let cold_speedup =
            reference["median_ms"].as_f64().unwrap() / cold["median_ms"].as_f64().unwrap();
        let case = json!({"name":name,"regex":regex,"states":work.states,"transitions":transition_count,
            "naive_byte_transitions":naive_bytes,"trie_byte_transitions":work.byte_transitions,
            "reference":reference,"cold_trie":cold,"warm_trie":warm,
            "cold_speedup":cold_speedup,"warm_speedup":warm_speedup,"exact_index_equality":true});
        eprintln!("{name}: cold {cold_speedup:.2}x, prepared {warm_speedup:.2}x");
        output["cases"].as_array_mut().unwrap().push(case);
        std::fs::write(&args[2], serde_json::to_vec_pretty(&output).unwrap()).unwrap();
    }
}
