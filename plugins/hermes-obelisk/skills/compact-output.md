# Obelisk Compact Output

Use `obelisk_run` for noisy read-heavy commands that would otherwise flood Hermes context.

## Good candidates

```text
git status
cargo build
cargo test
pytest
npm test
rg "TODO" src
```

## Refuse or avoid

```text
git push
rm -rf
sudo ...
commands with pipes or redirects
commands containing secrets or tokens
interactive editors
```

The plugin adapter already refuses many dangerous patterns, but do not try to outsmart it. Humans tried that with shell scripts and civilization never fully recovered.
