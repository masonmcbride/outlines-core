# Publish this application sample

The local checkout is ready on `trie-prefix-sharing`, with upstream history retained.
`origin` is `https://github.com/masonmcbride/outlines-core.git` and `upstream` is
`https://github.com/dottxt-ai/outlines-core.git`. No GitHub fork or Pages site has
been created by this preparation.

## Create the fork and push

Using GitHub CLI, if installed:

```sh
gh auth login --git-protocol https --scopes workflow
gh repo fork dottxt-ai/outlines-core --clone=false --remote=false
cd ~/w/cs/fun/outlines-core
git add Cargo.toml Cargo.lock .gitignore README.md PUBLISH.md src/index.rs src/index/trie.rs tests/trie_equivalence.rs examples/trie_benchmark.rs docs/trie .github/workflows/publish-trie.yml .github/workflows/test-trie.yml
git diff --cached --stat
git commit -m "Add optional vocabulary-prefix sharing with tests and measured exposition"
git push -u origin trie-prefix-sharing
gh repo edit masonmcbride/outlines-core --default-branch trie-prefix-sharing
```

Alternatively create the fork using GitHub's Fork button, then commit and push
the existing local checkout. Set its default branch to `trie-prefix-sharing`. Keep the repository name `outlines-core` to preserve
the prepared source links and Pages URL.

## Make the HTML readable

In the fork's **Settings → Pages**, choose **GitHub Actions** as the source.
Enable Actions for the fork if GitHub requests it. Under **Actions**, select
**Publish trie explanation**, choose `trie-prefix-sharing`, and run the workflow.
Verify the completed deployment before using this application link:

`https://masonmcbride.github.io/outlines-core/`

The repository's HTML file view shows source; the Pages deployment serves the
reading page. The explanation links to the implementation, tests, and notebook.

## Push the private resume repository

```sh
cd ~/w/cs/fun/resume-gen
gh repo view masonmcbride/resume-gen --json visibility
git status --short
git add -A
git diff --cached --stat
git commit -m "Preserve application records and extract Outlines research"
git push
```

Keep `resume-gen` PRIVATE; it contains candidate facts and feedback history.
The historical Outlines research is in `~/w/cs/fun/dottxt-research-archive.zip`,
with its original migration manifest and a separate hash receipt. It is separate from the public fork.
