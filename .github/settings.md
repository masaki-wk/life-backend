# Settings on GitHub

## General

- Default branch: `main`

## Rulesets

- `main`
  - Target branches: `main`
  - Bypass mode: Not permitted
  - Bypass list: (empty)
  - Branch protections
    - Restrict creations
    - Restrict deletions
    - Require a pull request before merging
      - Additional settings: (empty)
    - Require status checks to pass before merging
      - Require branches to be up to date before merging: enabled
      - Status checks that are required
        - verify-format
        - lint
        - verify-doc
        - test (ubuntu-latest, stable)
        - test (ubuntu-latest, nightly)
        - test (macos-latest, stable)
        - test (macos-latest, nightly)
        - test (windows-latest, stable)
        - test (windows-latest, nightly)
    - Block force pushes
