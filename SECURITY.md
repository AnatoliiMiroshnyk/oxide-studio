# Security policy

## Supported versions

Security fixes are applied to the latest supported release line. The current repository baseline is `0.1.x` and is a prototype, not a production security certification.

## Reporting a vulnerability

Please use GitHub's private **Report a vulnerability** flow for this repository when it is enabled. If private reporting is not enabled, contact the maintainers through the private contact channel published in the repository profile. Do not disclose vulnerability details publicly until a fix or mitigation is available.

Include the affected version, operating system, reproduction steps, impact assessment, and a minimal proof of concept. Do not attach real database files, customer data, credentials, or other secrets.

## Response expectations

Maintainers should acknowledge a report, assess severity and affected versions, coordinate a fix, and publish an advisory after a patch is available. Response timelines will be published when the project establishes a staffed maintenance process.

## Scope notes

Oxide Studio executes SQL against user-selected local SQLite databases. Arbitrary SQL execution is an intentional product feature; accidental data modification, unsafe import handling, memory exhaustion, plugin execution, and local file access boundaries are security-relevant areas.
