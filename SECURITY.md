# Security policy

An input method sees every key a person types, so we treat security reports
as top priority.

## What Buuz promises

- The input method makes no network connections.
- It does not log or transmit typed text.
- It stores only the personal dictionary (the words you add, the corrections
  you undo and how often you use known words). The dictionary stays on the
  device. You can see, delete and clear it. The core never writes it to
  disk: the shell does.
- Undoing a correction stores the word as you typed it. That word may not
  be in the word list: it can be a name.
- Learning is off by default in classic mode. A config setting can turn it
  on.
- Shells turn learning off for password fields and, where the platform tells
  them, for private windows.
- It does not run in password fields where the operating system marks them
  secure.

Anything that breaks one of these promises is a security bug.

## Reporting a vulnerability

Please do **not** open a public issue. Use GitHub's
[private vulnerability reporting](https://github.com/TengisCh/buuz/security/advisories/new).
We aim to acknowledge reports within 7 days and to ship a fix or mitigation
within 30 days for confirmed issues.

## Supported versions

Only the latest release receives security fixes during early development.
