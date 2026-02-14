# Project Policies

**Version**: v1.0  
**Last Updated**: 2026-02-13  
**Document Type**: Project Policy

---

## File Placement

- Do not add new files at the repository root.
- Put documentation under `docs/`, configurations under `config/`, scripts under `scripts/`, and data under `data/` or `dataset/`.
- Translation drafts and notes must go to `docs/translation-notes/` (ignored by git).

## Documentation Pairing

- All user-facing docs must exist in both `docs/en/` and `docs/zh/` with the same relative path and filename.
- When you update one language, update the other in the same change.
- New docs under `docs/` must live under `docs/en/` and `docs/zh/`, not directly under `docs/`.

## Enforcement

- Run `python3 scripts/verify_doc_pairs.py` or `./scripts/pre_commit_test.sh` before committing.
- CI and pre-commit checks may fail if these rules are violated.

## Exceptions

- Exceptions require maintainer approval and must be documented in the pull request.
