"""Execute the saved-data notebook and export the same explanation as HTML."""
import json
from pathlib import Path
import textwrap
import nbformat as nb
from nbclient import NotebookClient
from nbconvert import HTMLExporter

HERE = Path(__file__).resolve().parent


def md(text):
    return nb.v4.new_markdown_cell(textwrap.dedent(text).strip())


def code(text, hidden=False):
    cell = nb.v4.new_code_cell(textwrap.dedent(text).strip())
    if hidden:
        cell.metadata['tags'] = ['hide-input']
    return cell


def main():
    source = (HERE.parents[1]/'src/index/trie.rs').read_text()
    start = source.index('            stack.push((0, current_state));')
    end = source.index('\n            for (rank, candidate)', start)
    excerpt = textwrap.dedent(source[start:end]).strip()
    cells = [md(r"""
    # Read each token prefix once

    **Mason McBride · An Outlines Core experiment**

    Outlines compiles a constraint into an index: from each automaton state,
    which vocabulary tokens are valid, and where do they lead?

    The original constructor walks every token's bytes separately.
    Tokens share prefixes. Their walks can share computation too.
    """), code("""
    import hashlib, json, statistics
    from pathlib import Path
    from IPython.display import SVG, display
    from notebook_analysis import plot_results
    root = Path.cwd()
    provenance = json.loads((root/'SOURCE_PROVENANCE.json').read_text())
    for name, expected in provenance['measurements_sha256'].items():
        assert hashlib.sha256((root/name).read_bytes()).hexdigest() == expected
    datasets = {n: json.loads((root/f'data/{n}.json').read_text()) for n in ['gpt2','qwen','synthetic']}
    """, True), md(r"""
    ## Share the walk

    Take `return`, `returned`, and `returns`. Separate walks consume
    **6 + 8 + 7 = 21 bytes**. A trie consumes the shared `return` prefix
    once, then follows three suffix edges: **9 byte transitions**.
    These counts assume every path survives.
    """), code("display(SVG(filename=str(root/'figures/prefix_trie.svg')))", True),
    md(r"""
    Why is that safe? A deterministic automaton reaches the same state after
    the same prefix. Writing $\delta^*$ for a walk over bytes:

    $$\delta^*(q,xy)=\delta^*(\delta^*(q,x),y).$$

    Carry the current DFA state down each trie edge. Record the state at each
    token endpoint. If a prefix reaches a dead state, skip its entire subtree.

    ```text
    visit(node, state):
        record this state for tokens ending here
        for byte, child in node.children:
            next = DFA.next(state, byte)
            if next survives:
                visit(child, next)
    ```

    This illustrates the traversal. The Rust implementation uses an explicit
    stack, so a long token does not consume the call stack.
    """), md('<details><summary>The traversal — exact Rust source</summary>\n\n```rust\n'+excerpt+'\n```\n\n</details>'),
    md(r"""
    ## Preserve the behavior

    The trie stores **bytes**, because tokenizer entries may contain only part
    of a UTF-8 character. Tokens can also end inside another token's path.

    Traversal changes encounter order. The prototype restores the original
    vocabulary order before inserting destinations, preserving duplicate-ID
    overwrites and which error is reported first. That bookkeeping costs time.

    The original constructor stays intact. Tests compare complete indexes and
    exact errors across **3,000 generated cases**; a separate finite-language
    oracle checks valid continuations and EOS.
    """), md(r"""
    ## Pay for preparation

    A vocabulary trie can serve several constraints. Measure both ways:
    **build it for each index**, or **prepare it once and reuse it**.

    For Qwen's JSON record, construction falls from **322 ms to 128 ms**
    including preparation, or **99 ms** with the vocabulary trie reused.
    For a three-digit constraint, preparation overwhelms the savings.
    """), code("plot_results(datasets, root)", True), md(r"""
    Bars are medians of 21 samples on an **i7-12700K**; whiskers are 
    95% bootstrap intervals. Every variant rebuilds the DFA and index.
    Only the prepared variant reuses the vocabulary trie.

    Saved work is not the whole cost. On synthetic permissive text, sharing
    prefixes performs **2.82× fewer byte transitions**, yet prepared construction
    slows from **2.99 ms to 3.88 ms**. Traversal and bookkeeping can outweigh
    the transitions they remove.

    The useful design is an optional, reusable vocabulary preparation step.
    The measurements support particular constructor workloads; generation
    throughput and Python integration remain unmeasured.
    """), md(r"""
    <details><summary>Implementation, checks, and measurement details</summary>

    Nodes and edges are packed in vectors. Each terminal retains every associated
    token ID. Rank-indexed candidates preserve the reference's insertion order;
    the output index representation and default constructor are unchanged.

    Rust coverage includes empty and prefix tokens, duplicate IDs, malformed
    constraints, split UTF-8, a 20,000-byte token, and finite-language checks.
    Seventeen existing tests and six added test functions passed in the recorded
    experiment, plus two normalization tests. Formatting and Clippy passed.

    Six constraints were measured against GPT-2, Qwen, and a synthetic control.
    Each variant received 21 samples with rotating order after warmup. A release
    Rust 1.90.0 build ran on one pinned logical CPU; frequency was not locked and
    the host was shared. Normalization, equality checks, and counters were outside
    timings. Returned index destruction was excluded; temporary trie destruction
    was included in the one-off variant. Separate-process repetitions are saved.

    The Qwen trie estimates 20.23 MB of allocated vector payload; this excludes
    allocator metadata, temporary build storage, and traversal scratch. Peak
    memory was not measured. No universal adoption threshold is established.

    </details>
    """), md(r"""
    [Implementation](https://github.com/masonmcbride/outlines-core/blob/trie-prefix-sharing/src/index/trie.rs) ·
    [Tests](https://github.com/masonmcbride/outlines-core/blob/trie-prefix-sharing/tests/trie_equivalence.rs) ·
    [Notebook](Prefix_Sharing.ipynb) · [Reproduce](README.md) ·
    [Raw Qwen samples](data/qwen.json) · [Source hashes](SOURCE_PROVENANCE.json)

    Based on [Outlines Core `116a7c3`](https://github.com/dottxt-ai/outlines-core/tree/116a7c381b446814fdfc9b2b28bea7ca4f900aae).
    """)]
    notebook = nb.v4.new_notebook(cells=cells, metadata={'kernelspec':{'display_name':'Python 3','language':'python','name':'python3'}})
    NotebookClient(notebook, timeout=180, kernel_name='python3', resources={'metadata':{'path':str(HERE)}}).execute()
    nb.write(notebook, HERE/'Prefix_Sharing.ipynb')
    exporter = HTMLExporter()
    exporter.exclude_input = True
    body, _ = exporter.from_notebook_node(notebook)
    style = '<style>body{background:white!important}.jp-Notebook{max-width:960px;margin:32px auto!important;padding:0 20px!important}.jp-RenderedHTMLCommon{font:17px/1.65 system-ui!important;color:#26333d}.jp-RenderedHTMLCommon h1{font-size:34px}.jp-RenderedHTMLCommon h2{font-size:23px;margin-top:32px}.jp-InputPrompt,.jp-OutputPrompt{display:none!important}.jp-Cell-inputWrapper{margin-left:0!important}.jp-RenderedImage img,.jp-RenderedSVG svg{max-width:100%;height:auto}summary{cursor:pointer;font-weight:600}details{padding:12px 0;border-top:1px solid #ddd}pre{font-size:13px!important}a{color:#246b8f!important}</style>'
    (HERE/'index.html').write_text(body.replace('</head>',style+'</head>'))
    print('Executed notebook and exported docs/trie/index.html')


if __name__ == '__main__':
    main()
