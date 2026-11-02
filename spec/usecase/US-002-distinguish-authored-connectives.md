---
id: US-002
title: Distinguish authored connectives from synthesized ones
type: US
relationships: []
---
# US-002: Distinguish authored connectives from synthesized ones

## Story

**As a** consumer of admitted checked expressions
**I want** each boolean connective node to say whether the author wrote it or the checker introduced it
**So that** I can record decisions only for authored connectives without guessing from source locations.

## Context

A checker may compose a connective to express a derived ordering. That node has
no clause the author wrote, so decision recording applies only to authored
connectives.
