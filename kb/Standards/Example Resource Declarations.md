---
source: mixed
---
# Example Resource Declarations

Every runnable tutorial or scenario must say what kind of resources it really
touches. A reader should never have to inspect source code to discover that an
example is half live and half fixture.

The atelier default is **REAL LIVE** for anything that claims to exercise an
external provider, service, or Mosaic source-observation path. Local real
solvers, policy engines, and pure Rust evaluators are acceptable when the
example's point is explicitly local. Deterministic substitutes, canned
provider fixtures, recorded HTTP, `Stub*Provider`, `Mock*Backend`, and
`Fake*Backend` decision paths belong in `arena-tests`, not in the atelier
scenario gallery.

Atelier code must also stay provider-neutral at the LLM layer. Runnable
examples that need a live chat model use Manifold provider APIs for selection,
health probing, credentials, and backend calls. They must not hard-code a model
vendor's SDK, endpoint, model id, or API-key environment variable at the
showcase level.

Place a `## Resource Declaration` section near the run commands in each
example README. If the example prints a long walkthrough, print the same
declaration at runtime before the first result.

## Required Claims

Each declaration must include:

- **Trust label** — one of `REAL LIVE`, `LOCAL REAL`, `CONTRACT-SHAPE`,
  `SIMULATED`, or `MIXED`. Add a network qualifier such as
  `/ NO LIVE NETWORK` when it prevents confusion.
- **Live external resources** — yes or no. If yes, name the endpoints,
  services, or provider families actually called.
- **Mosaic extension mocking** — say whether atelier replaces any Mosaic
  extension with local mocks. If an extension's own `Stub*`, `Mock*`, or
  `Fake*` backend is configured, name it explicitly.
- **Solver/model backend** — name the actual solver, model provider, local
  fixture, or native FFI backend.
- **Credentials and env vars** — list the exact keys or feature flags needed
  for live behavior.
- **Trust boundary** — say what the example proves and what it does not prove.

## Labels

`REAL LIVE` means the run calls the named external services or providers
through the real provider path. For Mosaic examples, this means the real Mosaic
extension path all the way down to the live transport. It may not use `Stub*`,
`Mock*`, `Fake*`, canned fixtures, recorded HTTP responses, or local
substitutes on the decision path. If credentials are absent, the example must
exit honestly instead of falling back to a fake.

`LOCAL REAL` means the run uses real local implementation paths with no fake
decision backend. Examples include CVC5, OR-Tools, HiGHS, Cedar policy
evaluation, or pure Rust deterministic evaluators when those are the actual
product behavior. Use `/ NO LIVE NETWORK` when no external service is called.

`CONTRACT-SHAPE` means the run uses real Converge/Mosaic contracts and typed
facts, but at least one resource backend is deterministic, stubbed, or
fixture-backed. This is valid for `arena-tests` and upstream pressure tests.
It is not a finished atelier showcase scenario.

`SIMULATED` means the example is intentionally synthetic: no live provider,
solver, or external process changes the result. Simulations belong in
`arena-tests` unless they are part of the numbered learning spine and are
declared as local primitives, not external integrations.

`MIXED` means some paths are live and others are not. The declaration must name
which decision path is live and which is simulated. `MIXED` is a migration
label, not a target label for new atelier scenarios.

## Non-Negotiables

- Do not call an example "real" just because it imports a real Mosaic crate.
  The backend that decides the result is what matters.
- Do not hide an extension-provided stub behind "not mocked." If `Stub*Provider`
  or `Fake*Backend` is configured, say so.
- Do not silently fall back from live to fake behavior. Exit honestly.
- Do not claim live data freshness unless a live endpoint was actually called
  during that run.
- Do not land a new `scenarios/` example with a fake decision path. Put that
  coverage in `arena-tests` or implement the live upstream provider first.
- Do not encode LLM provider preference in atelier. Use Manifold selection and
  let deployment/operator configuration choose the provider.

## Template

```md
## Resource Declaration

**Trust label:** `REAL LIVE | LOCAL REAL | CONTRACT-SHAPE | SIMULATED | MIXED`.

- Live external resources: ...
- Mosaic extensions: ...
- Backend mode: ...
- Credentials / feature flags: ...
- Trust boundary: ...
```
