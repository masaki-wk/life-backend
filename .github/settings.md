# Settings on GitHub

## General

- Default branch: `main`

## Rulesets

- `All branches`
  - Target branches: All branches
  - Bypass mode: Not permitted
  - Bypass list: (empty)
  - Branch protections
    - Require status checks to pass before merging
      - Require branches to be up to date before merging: enabled
      - Status checks that are required: verify-format, lint, doc, test
- `main`
  - Target branches: `main`
  - Bypass mode: Not permitted
  - Bypass list: (empty)
  - Branch protections
    - Restrict creations
    - Restrict deletions
    - Require a pull request before merging
      - Additional settings: (empty)
    - Block force pushes
