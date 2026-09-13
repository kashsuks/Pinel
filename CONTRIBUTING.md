# Contributing to Pinel

## Running Tests

```bash
cargo test --all
```

Or use the helper script, which also records that tests passed for the current commit (see below):

```bash
./utils/run-tests.sh
```

For the broader code-quality suite (formatting, clippy, docs, etc, - not just tests) , see `./utils/clean.sh`.

## Git Hooks

This repo ships a tracked `pre-pushed` hook that blocks `git push` unless `./utils/run-tests.sh` has already passed for the commit you're pushing. Git nevery syncs `./git/hooks/` automatically, so it needs to be enabled once per clone:

```bash
git config core.hooksPath .githooks
```

After that, pushing without having run the tests since your last commit will fail with a reminder, rather than silently letting untested code go out. If you need to bypass it for some reason, `git push --no-verify` skips it for that one push.

Note that this hook is a local convenience, not a hard guarantee since it has to be manually enabled per clone, and can always be skipped with `--no-verify`. The actual enforcement is the `Tests` GitHub Actions workflow, which runs on every push and pull request regardless of any local setup.
