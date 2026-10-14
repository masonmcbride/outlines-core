//! Experimental vocabulary-prefix sharing during index construction.
//! The existing `Index::new` remains the reference implementation.

use super::*;

#[derive(Default)]
struct BuildNode {
    children: Vec<(u8, usize)>,
    terminal: Option<usize>,
}

struct Node {
    edge_start: usize,
    edge_len: usize,
    terminal: Option<usize>,
}

/// A packed byte trie reusable across regexes sharing the same vocabulary.
pub struct VocabularyTrie {
    nodes: Vec<Node>,
    edges: Vec<(u8, usize)>,
    token_ids: Vec<Vec<TokenId>>,
    eos_token_id: TokenId,
    vocab_size: usize,
}

/// Work counters collected outside benchmark timings.
#[derive(Default, Debug)]
pub struct TrieWork {
    pub states: usize,
    pub byte_transitions: usize,
    pub surviving_tokens: usize,
}

impl VocabularyTrie {
    pub fn new(vocabulary: &Vocabulary) -> Self {
        // Keep terminal ranks in original vocabulary order. Traversing a trie
        // changes encounter order; restoring it preserves duplicate-ID overwrite
        // semantics and which incompatible state the reference reports first.
        let mut ordered = Vec::with_capacity(vocabulary.tokens().len());
        let mut token_ids = Vec::with_capacity(vocabulary.tokens().len());
        for (token, ids) in vocabulary.tokens() {
            if ids.contains(&vocabulary.eos_token_id()) {
                continue;
            }
            ordered.push((token.as_slice(), token_ids.len()));
            token_ids.push(ids.clone());
        }
        ordered.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut nodes = vec![BuildNode::default()];
        let mut path = vec![0];
        let mut previous: &[u8] = &[];
        for (token, rank) in ordered {
            let common = previous
                .iter()
                .zip(token)
                .take_while(|(a, b)| a == b)
                .count();
            path.truncate(common + 1);
            for &byte in &token[common..] {
                let parent = *path.last().unwrap();
                let child = nodes.len();
                nodes.push(BuildNode::default());
                nodes[parent].children.push((byte, child));
                path.push(child);
            }
            nodes[*path.last().unwrap()].terminal = Some(rank);
            previous = token;
        }

        let mut packed = Vec::with_capacity(nodes.len());
        let mut edges = Vec::with_capacity(nodes.len().saturating_sub(1));
        for node in nodes {
            packed.push(Node {
                edge_start: edges.len(),
                edge_len: node.children.len(),
                terminal: node.terminal,
            });
            edges.extend(node.children);
        }
        Self {
            nodes: packed,
            edges,
            token_ids,
            eos_token_id: vocabulary.eos_token_id(),
            vocab_size: vocabulary.len(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Allocated payload estimate; excludes allocator metadata and build scratch.
    pub fn estimated_heap_bytes(&self) -> usize {
        self.nodes.capacity() * std::mem::size_of::<Node>()
            + self.edges.capacity() * std::mem::size_of::<(u8, usize)>()
            + self.token_ids.capacity() * std::mem::size_of::<Vec<TokenId>>()
            + self
                .token_ids
                .iter()
                .map(|v| v.capacity() * std::mem::size_of::<TokenId>())
                .sum::<usize>()
    }

    pub fn profile_build(&self, regex: &str) -> Result<(Index, TrieWork)> {
        self.build::<true>(regex)
    }

    fn build<const COUNT: bool>(&self, regex: &str) -> Result<(Index, TrieWork)> {
        let dfa = DFA::new(regex).map_err(Box::new)?;
        let start_state = dfa
            .universal_start_state(Anchored::Yes)
            .ok_or(Error::DfaHasNoStartState)?;
        let mut transitions: HashMap<StateId, HashMap<TokenId, StateId>> = HashMap::default();
        let mut final_states = HashSet::default();
        let mut seen = HashSet::from_iter([start_state]);
        let mut next_states = vec![start_state];
        let mut useful_cache: HashMap<AutomataStateId, bool> = HashMap::default();
        let mut candidates = vec![None; self.token_ids.len()];
        let mut stack = Vec::new();
        let mut work = TrieWork::default();

        while let Some(current_state) = next_states.pop() {
            if COUNT {
                work.states += 1;
            }
            let mut has_valid_transitions = false;
            if dfa.is_match_state(dfa.next_eoi_state(current_state)) {
                final_states.insert(current_state.as_u32());
                has_valid_transitions = true;
            }

            candidates.fill(None);
            stack.push((0, current_state));
            while let Some((node_id, state)) = stack.pop() {
                let node = &self.nodes[node_id];
                if let Some(rank) = node.terminal {
                    candidates[rank] = Some(state);
                    if COUNT {
                        work.surviving_tokens += 1;
                    }
                }
                for &(byte, child) in &self.edges[node.edge_start..node.edge_start + node.edge_len]
                {
                    let next = dfa.next_state(state, byte);
                    if COUNT {
                        work.byte_transitions += 1;
                    }
                    if !dfa.is_dead_state(next) && !dfa.is_quit_state(next) {
                        stack.push((child, next));
                    }
                }
            }

            for (rank, candidate) in candidates.iter().enumerate() {
                let Some(next_state) = *candidate else {
                    continue;
                };
                let useful = *useful_cache.entry(next_state).or_insert_with(|| {
                    dfa.is_match_state(dfa.next_eoi_state(next_state))
                        || dfa.byte_classes().representatives(..).any(|repr| {
                            repr.as_u8().is_some_and(|byte| {
                                let s = dfa.next_state(next_state, byte);
                                !dfa.is_dead_state(s) && !dfa.is_quit_state(s)
                            })
                        })
                });
                if useful {
                    has_valid_transitions = true;
                    for token_id in &self.token_ids[rank] {
                        transitions
                            .entry(current_state.as_u32())
                            .or_default()
                            .insert(*token_id, next_state.as_u32());
                    }
                    if !seen.contains(&next_state) {
                        seen.insert(next_state);
                        next_states.push(next_state);
                    }
                }
            }

            if !has_valid_transitions && !dfa.is_match_state(current_state) {
                let mut valid_characters = Vec::new();
                for byte in 0..=255u8 {
                    let state = dfa.next_state(current_state, byte);
                    if !dfa.is_dead_state(state) && !dfa.is_quit_state(state) {
                        valid_characters.push(if byte.is_ascii() {
                            char::from(byte).to_string()
                        } else {
                            format!("\\x{:02x}", byte)
                        });
                    }
                }
                return Err(Error::IncompatibleVocabulary {
                    regex: regex.to_string(),
                    error_state: current_state.as_u32(),
                    missing_tokens: valid_characters,
                });
            }
        }
        for &state in &final_states {
            transitions
                .entry(state)
                .or_default()
                .insert(self.eos_token_id, state);
        }
        Ok((
            Index {
                initial_state: start_state.as_u32(),
                final_states,
                transitions,
                eos_token_id: self.eos_token_id,
                vocab_size: self.vocab_size,
            },
            work,
        ))
    }
}

impl Index {
    /// Experimental one-off construction, including the cost of building a trie.
    pub fn new_with_trie(regex: &str, vocabulary: &Vocabulary) -> Result<Self> {
        Self::new_with_prepared_trie(regex, &VocabularyTrie::new(vocabulary))
    }

    /// Experimental construction reusing a previously prepared vocabulary trie.
    pub fn new_with_prepared_trie(regex: &str, trie: &VocabularyTrie) -> Result<Self> {
        trie.build::<false>(regex).map(|(index, _)| index)
    }
}
