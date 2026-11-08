# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-eval's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The hero follows
crates/quire-eval/src/session.rs and registry.rs.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("model", "checked closure", "box"),
    ("logic", "evaluation core", "box"),
    ("evidence", "result", "box"),
    ("ink", "meter", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: a Session stepping a CheckedClosure through a Registry of bodies."""
    f = Figure(
        "hero",
        980,
        300,
        "A Session steps a CheckedClosure one transition at a time. The Registry supplies the "
        "resumable host bodies and the closure advances interpreted code. Frames sit on one heap "
        "task stack, each step charges a Meter, and a step returns Runnable or a Terminal result.",
    )
    f.box(0, 30, 200, 56, "logic", "Registry", "resumable host bodies")
    f.box(0, 150, 200, 56, "model", "CheckedClosure", "interpreted callables")
    f.box(300, 90, 200, 68, "logic", "Session", "step() · one transition", "owns the task stack")
    f.line([(200, 58), (270, 58), (270, 112), (297, 112)], "logic")
    f.line([(200, 178), (270, 178), (270, 136), (297, 136)], "model")
    f.note(206, 50, "bodies", "muted")
    f.note(206, 170, "advance", "muted")

    f.text(600, 20, "HEAP TASK STACK", "muted", "mono", SMALL)
    f.box(600, 30, 190, 56, "logic", "callee frame", "top of stack")
    f.box(600, 100, 190, 56, "ink", "caller frame", "suspended")
    f.line([(500, 110), (550, 110), (550, 58), (597, 58)], "logic")
    f.note(506, 102, "push", "muted")

    f.box(300, 230, 200, 56, "ink", "Meter", "work_units fuel")
    f.line([(400, 158), (400, 227)], "logic")
    f.note(412, 198, "charges", "muted")

    f.box(600, 200, 190, 56, "evidence", "Progress", "Runnable · Terminal")
    f.line([(500, 146), (550, 146), (550, 228), (597, 228)], "logic")
    f.box(830, 200, 150, 56, "evidence", "Evaluation", "or CallFailure")
    f.line([(790, 228), (827, 228)], "evidence")
    f.note(600, 282, "Runnable: step again", "muted")
    return f


# (column, row, repository, what it does, private). Rows: 0 orchestration, 1 the
# main chain, 2-3 branches off it. Arrows follow what one repository's output feeds.
REPOS = [
    (1, 0, "quire-driver", "the quire command", False),
    (2, 0, "quire-integration", "tests the whole chain", False),
    (3, 0, "quire-verification", "plans verification", False),
    (0, 1, "quire-specification", "the standard", False),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", False),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (3, 2, "quire-eval", "evaluation core", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-spec-language"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. quire-eval is the evaluation core. Above the chain, quire-driver runs it, "
        "quire-integration tests it and quire-verification plans verification. Below it, shared "
        "libraries used throughout: quire-exact, quire-semantic-value, quire-walk, "
        "quire-canonical and quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("model", "feeds", "fill")])
    return f


def ecosystem_here():
    """The ecosystem figure with quire-eval highlighted."""
    return ecosystem(here="quire-eval")


FIGURES = [hero, ecosystem_here]
