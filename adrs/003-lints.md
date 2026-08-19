# ADR-003: Lints

## Status

Accepted

## Context

[ADR-001][adr-001] records why we make a decision, and [ADR-002][adr-002]
records what a crate must do. Neither document guards how the code is written.
That is the role of our engineering rules: the conventions for style,
structure, errors, dependencies, and tests that `AGENTS.md` describes in
prose.

Prose does not enforce itself. Coding agents write most of the code, and they
follow prose rules unreliably. When an agent misses a rule, a reviewer must
find the violation by hand, and the same violation returns in the next change.
A rule that nothing enforces is a suggestion.

Two mechanisms can enforce a rule. A deterministic check, such as a formatter
or a lint, gives the same verdict on every run and does not tire. A
non-deterministic review, by a human or by an AI reviewer, can judge
properties that no algorithm can decide, but its verdict varies and its
attention is scarce. Guardrails need both, and the split between them is a
decision.

## Decision

We enforce our engineering rules with a mix of deterministic checks and
non-deterministic reviews. We codify every rule that a machine can decide,
and we treat every review comment as a candidate for a new check.

1. **Checks are the first line.** Every rule that a machine can decide
   becomes a check: formatters, linters, license and dependency audits, the
   specification check, and the tests. Each check is a `just` recipe.
   `just pre-commit` runs the checks locally, and CI runs the same recipes.
   A rule that a check enforces never needs a review comment.

2. **Reviews judge what checks cannot.** A review asks whether a change fits
   the ADRs, whether it implements the right requirements, and whether names
   and documentation are clear. Both humans and AI reviewers review changes.
   A review does not repeat the checks. When a reviewer comments on
   formatting or style, that is a signal that a check is missing, not that
   the review must look closer.

3. **Every review comment questions the checks.** When a review finds an
   issue, we ask whether a check could have found it mechanically. If it
   could, we enable a lint, configure a rule, or write a custom recipe in the
   same way we fix the code. The comment removes one instance; the check
   removes the class. Until a rule has a check, the reviews carry it, and
   `AGENTS.md` states it in prose.

4. **Checks block, they do not advise.** A check either passes or fails, and
   a failed check blocks the commit and the merge. Warnings that do not fail
   accumulate until everybody ignores them, so the linters deny warnings.
   When a check is wrong or too strict, we change or delete the check. We do
   not silence it case by case without a comment that explains why.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### Rules in Prose with Reviews Only

`AGENTS.md` can state every rule, and reviews can enforce them. This needs no
tooling, but it does not scale. Every rule competes for the limited attention
of a reviewer, and the mechanical rules crowd out the questions of design.
Agents and humans drift from prose rules, and the same comment repeats in
every pull request. Review attention is the most expensive resource we have,
and we do not want to spend it on what a machine can decide.

### Checks Only

We can try to codify every rule and drop reviews. But the properties that
matter most are not decidable: whether a design fits an ADR, whether a
requirement is right, whether a name misleads. A check for such a property
guesses, and a check that guesses produces false positives until it is
ignored. Reviews stay, and the checks exist to keep them short.

### Advisory Warnings

Checks can warn without failing, and a person can weigh each warning. This
avoids friction on the first violation, but warnings without consequence
accumulate, and a wall of stale warnings hides the new one that matters. A
rule either gates or it does not exist. When a rule is not worth a failure,
we remove the rule instead of demoting it to a warning.

## Consequences

- Guardrails come in three layers: ADRs record why, specifications record
  what, and checks with reviews enforce how. A new contributor, human or
  agent, learns the rules from the checks that fail, not only from prose.
- The check suite grows with every review. A class of issue that a review
  found once does not return, and the reviews get shorter and closer to
  design over time.
- Codifying a rule costs effort, and a check must be worth its false
  positives. A too strict check adds friction to every commit, so deleting a
  bad check is a normal change, not a defeat.
- Blocking checks stop a commit on every violation. Agents get a fast and
  deterministic feedback loop from `just pre-commit` instead of a slow and
  variable one from review.
- The time to run the checks grows with the suite, locally and in CI. We
  keep the pre-commit checks parallel, and a check that gets too slow moves
  to CI only.
- Some rules stay prose for a long time, because no linter decides them and
  a custom check is not worth its cost yet. The reviews carry these rules,
  and the pressure to codify them stays.

[adr-001]: 001-adrs.md
[adr-002]: 002-specifications.md
