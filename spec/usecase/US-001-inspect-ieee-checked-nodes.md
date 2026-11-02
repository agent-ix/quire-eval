---
id: US-001
title: Inspect admitted IEEE operations through checked nodes
type: US
relationships: []
---
# US-001: Inspect admitted IEEE operations through checked nodes

## Story

**As a** consumer of admitted checked expressions
**I want** to identify each IEEE operation and inspect its operands through a typed read
**So that** I can use the admitted meaning without parsing the wire form again.

## Context

The contract-ir operation catalogue distinguishes fused multiply-add by width,
IEEE total order, and bit identity. A checked expression reader needs to retain
those distinctions for downstream consumers.
