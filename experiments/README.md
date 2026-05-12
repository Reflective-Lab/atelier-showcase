# Experiments

This directory holds falsifiable showcase experiments.

Experiments in atelier are not foundation contracts and not product
commitments. They are worked probes that connect a capability from the
Reflective Stack to a concrete scenario, then record whether the integration
was actually useful, awkward, or unnecessary.

## Rules

- Anchor every experiment in an existing `scenarios/` pattern unless the
  experiment is explicitly about adding a new scenario.
- Keep lower-stack claims domain-neutral. Domain and business language belongs
  here or in applications built on the stack, not in foundation layers.
- State a hypothesis, a falsification path, and the smallest artifact that
  would make the result useful.
- Treat friction as data. If the experiment needs source-code archaeology,
  undocumented schema knowledge, or a lower-layer patch, record it.
- Prefer synthetic but realistic fixtures until a real engagement or app pulls
  the experiment forward.

## Index

- [EXP-001: Fuzzy Interpretation for Vendor Selection](EXP-001-fuzzy-interpretation.md)
