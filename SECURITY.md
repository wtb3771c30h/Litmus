# Security Policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private
[security advisory](https://github.com/wtb3771c30h/Litmus/security/advisories/new) form, or contact a
maintainer directly if the form is unavailable to you.

Include what an attacker gains in one sentence, the version or commit you are running, the smallest
reproduction you can manage, and whether the input is attacker-controlled in your setup. You should
get an acknowledgement within a few working days and a disclosure timeline before anything is
published.

## Scope

Litmus runs a command repeatedly and fingerprints the outcomes. In scope: a scenario
whose fingerprint changes between identical runs, a normalization rule that hides a
real divergence, a crafted scenario file (unicode, huge output) that crashes the
runner, and path handling in the report writer.
