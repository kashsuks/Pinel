# Git Conventions

## Commit Rules

- Do not commit code automatically unless explicitly requested
- Ensure the code runs correctly before committing
- If already in a branch commit there, else, commit in the main/master branch

## Commit Message Format

```
<type>[<scope>]: <subject>
```

A space follows the colon. Type values:

| type | Purpose |
|------|---------|
| feat | New feature |
| fix | Bug fix |
| docs | Documentation or comments |
| style | Code formatting (no runtime impact) |
| refactor | Refactoring (not a new feature or bug fix) |
| perf | Performance optimization |
| test | Adding tests |
| chore | Build process or tooling changes |

Additionally, ensure that the first letter of type, scope, and subject are capitalized.

## Squash Commits and Pull Requests

PR Message/Title Format

PR's MUST use squash commit formatting as follows

```
[Scope] <overview of changes for this PR>
```

Note that the first letter of the scope and commit message must be capitalized

For example:

```
[Shooter] Add different shooting angles
```

This allows us to squash multiple commits that were in the Pull Request into one general commit.

## Other

- You MUST also use squash commit formatting if your commit covers more than one changed issue. Refer to the [squash commit](#squash-commits-and-pull-requests)
- Keep commit names short and elaborate. They're meant to be used to quickly identify issues or to understand changes.
