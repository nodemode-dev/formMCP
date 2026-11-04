# Security policy

## Supported versions

Security fixes go into the newest release. Please update before reporting, and
check whether the problem is still there.

| Version | Supported |
| --- | --- |
| 0.7.x (latest release) | Yes |
| Anything older | No |

## Reporting a vulnerability

**Do not open a public issue for a security problem.** Use GitHub's private
reporting instead: on the repository's **Security** tab, choose **Report a
vulnerability**. Include what you found, the steps to reproduce it, the version and
platform, and what an attacker could do with it.

You can expect an acknowledgement within a week. We will keep you informed while it
is fixed and credit you in the release notes, unless you would rather not be named.

## What counts

FormMCP runs on your own machine and starts other Form apps for your agents, so the
things that matter most are the ways something outside it could get in:

- **The HTTP server** (port 5625 by default, while the window or `formmcp serve` runs)
  binds to loopback (127.0.0.1). Anything that makes it reachable from another machine,
  or gets a request past the access token, the Host check or the Origin check.
- **Secrets** kept in the system keychain for an app's server: anything that leaks
  them to a file, the log, a result, a hook, a span or another app.
- **Starting apps**: getting FormMCP to start an installed app whose files differ from
  the record it checks against, or to start anything other than a Form app's own
  server.
- **What an agent is allowed**: an agent reaching an app or a tool that was turned off
  for it, or a before hook's refusal being skipped.
- **The family list**: getting FormMCP to trust a catalogue that was not signed with a
  key it carries.

Bugs that need someone to already control your machine, or to run a modified build
of FormMCP, are ordinary bugs: please open an issue for those.
