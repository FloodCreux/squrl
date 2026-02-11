# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in squrl, please report it responsibly by emailing the maintainer directly rather than opening a public issue.

Include:
- A description of the vulnerability
- Steps to reproduce
- Potential impact

You can expect an initial response within 7 days.

## Scope

squrl is a terminal HTTP client. Security-relevant areas include:

- **Credential handling** -- authentication credentials (Basic, Bearer, JWT, Digest) are stored in collection files in the working directory. Protect these files with appropriate filesystem permissions.
- **Pre/post scripts** -- JavaScript execution via the embedded Boa runtime. Scripts run with the same privileges as the squrl process.
- **Proxy configuration** -- proxy settings in `squrl.toml` route traffic through the specified servers. Verify proxy URLs before use.
- **TLS** -- squrl uses rustls via reqwest. No system OpenSSL dependency.

## Best Practices

- Do not commit collection files containing secrets to version control.
- Use environment variables (`{{variable}}` substitution) to keep credentials out of collection files.
- Review imported Postman collections, cURL files, and OpenAPI specs before executing requests, as they may contain unexpected URLs or scripts.
