# Security Policy

## Supported Versions

`gig` has no published releases yet - security fixes apply to the latest commit on `main`.

## Reporting a Vulnerability

Please do not open a public issue for security vulnerabilities.

Instead, report privately via [GitHub Security Advisories](https://github.com/merikan/git-get/security/advisories/new), or email peter@merikan.com.

Include:

- A description of the vulnerability and its impact.
- Steps to reproduce, or a proof-of-concept.
- Affected commit/version, if known.

You should get a response within a few days. This is a single-maintainer project, so there's no fixed SLA - if you haven't heard back after a week, feel free to follow up.

## Scope

`gig` shells out to your system's `git` binary and delegates authentication to your existing `ssh-agent`/git credential setup - it doesn't handle credentials itself. Relevant reports include things like:

- Path traversal or unsafe destination resolution (`root-dir` escapes, category routing, symlink handling).
- Command injection via URLs, category patterns, or other user-controlled input passed to `git` or the shell.
- Unsafe handling of untrusted repo content or config.

## Disclosure

Once a fix is available, it'll be released and noted in the commit history / release notes, with credit to the reporter unless anonymity is requested.
