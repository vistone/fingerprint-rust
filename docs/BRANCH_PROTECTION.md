# Branch Protection Setup Guide

## Overview

This document explains how to configure GitHub branch protection rules to enforce mandatory testing before merge.

## Why Branch Protection?

Branch protection ensures:
- ✅ All tests must pass before merge
- ✅ No direct commits to protected branches
- ✅ Code review requirements
- ✅ Status checks must pass
- ✅ Maintains code quality

## Required Setup

### 1. Navigate to Branch Protection Settings

1. Go to your repository on GitHub
2. Click **Settings** → **Branches**
3. Click **Add rule** or **Edit** existing rule
4. Set **Branch name pattern**: `main` (or `master`)

### 2. Configure Protection Rules

#### ✅ Require Pull Request Reviews

```
☑ Require a pull request before merging
  ☑ Require approvals: 1
  ☐ Dismiss stale pull request approvals when new commits are pushed
  ☑ Require review from Code Owners (if you have CODEOWNERS file)
```

#### ✅ Require Status Checks

**CRITICAL: Enable these checks**

```
☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging
  
  Required status checks (search and add these):
  ✅ All Required Checks Passed
  ✅ All Tests (Required)
  ✅ Format Check (Required)
  ✅ Clippy Check (Required)
  ✅ Build Check (Required) - ubuntu-latest
  ✅ Build Check (Required) - windows-latest
  ✅ Build Check (Required) - macos-latest
  ✅ Security Audit (Required)
```

#### ✅ Additional Protections

```
☑ Require conversation resolution before merging
☑ Require linear history (optional but recommended)
☐ Allow force pushes (keep UNCHECKED)
☐ Allow deletions (keep UNCHECKED)
```

### 3. Apply to Additional Branches

Repeat the above for:
- `develop` branch
- `master` branch (if used)
- Any release branches

## Status Checks Reference

### From `required-checks.yml`

These are the mandatory checks enforced by the `required-checks.yml` workflow:

1. **All Tests (Required)**
   - Runs all workspace tests
   - Runs doc tests
   - Runs AI models specific tests
   - **Must pass**: Cannot merge if any test fails

2. **Format Check (Required)**
   - Validates code formatting with `cargo fmt`
   - **Must pass**: All code must be formatted

3. **Clippy Check (Required)**
   - Runs `cargo clippy` with `-D warnings`
   - **Must pass**: Zero warnings allowed

4. **Build Check (Required)**
   - Builds on Ubuntu, Windows, and macOS
   - **Must pass**: Must build on all platforms

5. **Security Audit (Required)**
   - Runs `cargo-deny` security checks
   - **Must pass**: No vulnerabilities allowed

6. **All Required Checks Passed**
   - Summary check that depends on all above
   - **Must pass**: This is the final gate

## Verification

### Test Branch Protection

1. Create a test PR with a failing test
2. Verify that merge button is blocked
3. Verify status checks show as required
4. Fix the issue and verify checks pass
5. Verify merge button becomes available

### Expected Behavior

When branch protection is properly configured:

#### ❌ Failed Checks (Merge Blocked)
```
Some checks were not successful
1 failing and 5 successful checks

❌ All Required Checks Passed — Failed
  ↳ One or more required checks failed
  
☐ Merge pull request (blocked)
   Required status checks have not passed
```

#### ✅ Passed Checks (Merge Allowed)
```
All checks have passed

✅ All Required Checks Passed — Success
✅ All Tests (Required) — Success  
✅ Format Check (Required) — Success
✅ Clippy Check (Required) — Success
✅ Build Check (Required) - ubuntu-latest — Success
✅ Build Check (Required) - windows-latest — Success
✅ Build Check (Required) - macos-latest — Success
✅ Security Audit (Required) — Success

✓ Merge pull request
```

## For Repository Owners

### Initial Setup Checklist

- [ ] Create branch protection rule for `main`
- [ ] Add all required status checks
- [ ] Enable "Require pull request reviews"
- [ ] Test with a dummy PR
- [ ] Document in README.md
- [ ] Notify team members

### Maintenance

- Review protection rules quarterly
- Update required checks when workflows change
- Monitor for bypass attempts
- Review and update CODEOWNERS file

## For Contributors

### Before Creating a PR

Run locally to avoid CI failures:

```bash
# 1. Format code
cargo fmt --all

# 2. Check clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Run tests
cargo nextest run --workspace --all-features

# 4. Or use pre-commit hook
.git/hooks/pre-commit
```

### PR Workflow

1. Create feature branch
2. Make changes
3. Run local checks (above)
4. Push to GitHub
5. Create PR
6. Wait for all checks to pass
7. Request review
8. Merge after approval

## Troubleshooting

### Issue: Status Checks Not Showing

**Solution**: 
- Checks only appear after first workflow run
- Push a commit to trigger workflows
- Check Actions tab for workflow execution

### Issue: Can't Find Status Check Name

**Solution**:
- Look in Actions tab for exact check names
- Check names are defined in workflow files
- Use the "job name" from workflow YAML

### Issue: Merge Button Still Available Despite Failures

**Solution**:
- Branch protection not enabled correctly
- Admin/owner may have bypass permissions
- Re-check branch protection settings

### Issue: Pre-commit Hook Not Working

**Solution**:
```bash
# Re-install and make executable
cp .github/pre-commit-hook.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Test it
.git/hooks/pre-commit
```

## Integration with Existing Workflows

The `required-checks.yml` workflow complements existing workflows:

- **ci.yml**: Comprehensive testing across platforms
- **comprehensive-testing.yml**: Extended test suite
- **ai-models-validation.yml**: AI-specific validation
- **coverage.yml**: Coverage reporting
- **security-audit.yml**: Security scanning

All these workflows run, but **required-checks.yml** contains the mandatory gates for merge.

## Badge for README

Add this badge to show protection status:

```markdown
![Branch Protection](https://img.shields.io/badge/branch-protected-success)
![Required Checks](https://github.com/vistone/fingerprint-rust/workflows/Required%20Checks/badge.svg)
```

## References

- [GitHub Branch Protection Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/about-protected-branches)
- [Required Status Checks](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/about-protected-branches#require-status-checks-before-merging)
- [GitHub Actions Status Checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks)

---

**Last Updated**: 2026-02-16
