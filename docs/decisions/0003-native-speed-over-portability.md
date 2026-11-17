# 0003: Native speed over portability

Status: Accepted
Date: 2026-11-17

## Context

An input method sits on every keystroke. Latency and a snappy feel matter more
than sharing UI code across platforms.

## Decision

Where speed and portability conflict, choose speed. The typing path is native,
offline, allocation-light, and never logs keystrokes. Portable layers (such as
the web build) must not shape the core's design.

## Consequences

- Separate native shells per platform instead of one cross-platform UI.
- Changes to the typing path are checked against the performance budget in
  `docs/architecture.md`.
