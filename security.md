# Security Policy

Harbor is alpha software that handles sensitive wallet data and funds. Please
compile and use it with care, and do not store funds you cannot afford to lose.

## Supported Versions

Harbor does not currently publish stable releases. Security updates are made on
the default branch only; older commits and forks are not supported.

## Reporting a Vulnerability

Please do not open a public issue or discussion for a suspected vulnerability.
Instead, report it privately through [GitHub's security advisory
form](https://github.com/HarborWallet/harbor/security/advisories/new).

Include as much of the following as possible:

- A description of the vulnerability and its potential impact
- The affected commit, component, platform, and configuration
- Steps or a proof of concept that reproduce the issue
- Any suggested mitigations or fixes
- Whether the vulnerability has been disclosed elsewhere

Please avoid accessing other users' data, disrupting services, or moving funds
that do not belong to you while investigating. Use test networks and test funds
whenever possible.

The maintainers will review the report, work with you to understand and
validate it, and coordinate disclosure after a fix or mitigation is available.
We ask that you keep the report confidential until that process is complete.

## Scope

Reports about Harbor's own code and build or release process are in scope.
Vulnerabilities in upstream dependencies, ecash mints, Bitcoin or Lightning
infrastructure, Tor, operating systems, or other third-party services should be
reported to the relevant project. If an upstream issue has a specific security
impact on Harbor, please also notify the Harbor maintainers privately.
