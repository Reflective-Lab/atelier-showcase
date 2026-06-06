---
tags: [quality, migration, hermeticity, dependency-injection]
property: RP-HERMETIC-UNIT
incident: QF-2026-06-02-05
applies-to: any function that internally reads env or opens a connection
source: human + LLM
---

# Migration — Inject the backend selector

A walkable, copy-pasteable refactor for the most common
`RP-HERMETIC-UNIT` violation: a function that internally reads
`from_env()` or constructs a network client, and a test that can
only control its behavior by mutating dev-machine env.

Worked example: `bedrock-platform/axiom@0.15.1` → `0.15.2`,
commit `3fbe4fe`.

## Symptom you'll recognize

A test like:

```rust
#[tokio::test]
async fn does_the_right_thing_with_no_backend() {
    let config = GuidanceConfig::default();
    let result = guide_heading("Truth: …", &config).await;
    assert_eq!(result.unwrap().source, "local-heuristic");
}
```

…paired with a function that reads env internally:

```rust
async fn select_backend(cfg: &GuidanceConfig) -> Result<…, String> {
    let selection = ChatBackendSelectionConfig::from_env()?;
    let backend = select_healthy_chat_backend(&selection).await?;
    // …
}
```

The test "works" only on machines where the env has no creds. The
moment a developer sources an `.envrc` with `OPENAI_API_KEY`, the
test fails *and* burns API credit.

## The fix in five steps

### Step 1 — Name the seam

Identify the **smallest piece of env- or network-dependent
behavior** the function does. In our case: "pick a backend, or
return an error if none is available." That's a single async
fallible operation. Name it.

```rust
pub trait BackendSelector {
    fn select(
        &self,
        config: &GuidanceConfig,
    ) -> impl std::future::Future<Output = Result<SelectedBackend, String>>;
}
```

The trait method **is** the dependency-injection point.

### Step 2 — Make the production behavior an explicit implementor

Don't delete the env-reading code; relocate it. The behavior is
fine; it just needed a name.

```rust
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvBackendSelector;

impl BackendSelector for EnvBackendSelector {
    async fn select(&self, config: &GuidanceConfig) -> Result<SelectedBackend, String> {
        let mut selection = ChatBackendSelectionConfig::from_env()
            .map_err(|e| format!("ChatBackend selection configuration failed: {e}"))?;
        if let Some(provider) = &config.provider_override {
            selection = selection.with_provider_override(provider.clone());
        }
        let selected = select_healthy_chat_backend(&selection)
            .await
            .map_err(|e| format!("No live chat backend is available: {e}"))?;
        Ok(SelectedBackend {
            backend: selected.backend,
            provider: selected.provider().to_string(),
            model: selected.model().to_string(),
        })
    }
}
```

This is **exactly the old function body**. Nothing semantic
changed in production.

### Step 3 — Add the deterministic test implementor

```rust
#[derive(Debug, Default, Clone, Copy)]
pub struct NoBackendSelector;

impl BackendSelector for NoBackendSelector {
    async fn select(&self, _config: &GuidanceConfig) -> Result<SelectedBackend, String> {
        Err("backend selection disabled by NoBackendSelector".into())
    }
}
```

Note the **load-bearing error string**. The test will assert on it
to prove the fallback path fired *because the test wanted it to*,
not because the network was down for some other reason.

### Step 4 — Add a `*_with` variant that takes the selector

```rust
pub async fn guide_heading_with<S: BackendSelector>(
    spec: &str,
    config: &GuidanceConfig,
    selector: &S,
) -> Option<GuidanceResponse> {
    let current_title = extract_title(spec)?;
    let response = match request_live_guidance(spec, &current_title, config, selector).await {
        Ok(r) => r,
        Err(e) => local_heading_guidance(
            spec,
            &current_title,
            format!("Live guidance failed, showing local rewrite: {e}"),
        ),
    };
    Some(response)
}
```

The original `guide_heading(spec, config)` is preserved as a
convenience wrapper:

```rust
pub async fn guide_heading(spec: &str, config: &GuidanceConfig) -> Option<GuidanceResponse> {
    guide_heading_with(spec, config, &EnvBackendSelector).await
}
```

External consumers of the library don't break. Internal callers
that need determinism reach for `guide_heading_with` explicitly.

### Step 5 — Rewrite the test to **state** the no-backend premise

```rust
#[tokio::test]
async fn guide_heading_falls_back_to_local_on_no_backend() {
    let config = GuidanceConfig::default();
    let spec = "Truth: Vendor selection for AI rollout\n\nScenario: test\n  Given x";
    let result = guide_heading_with(spec, &config, &NoBackendSelector).await;
    assert!(result.is_some());
    let resp = result.unwrap();
    assert_eq!(resp.source, "local-heuristic");
    assert!(resp.note.contains("Live guidance failed"));
    assert!(
        resp.note.contains("NoBackendSelector"),
        "note should surface the selector's error so callers can diagnose: {}",
        resp.note
    );
}
```

The test now:
- **Names its precondition** in the code (`&NoBackendSelector`).
- **Reads nothing from env.**
- **Makes no network calls.**
- **Asserts the selector's specific error string** is plumbed
  through, which prevents a future regression where the fallback
  path silently fires for the wrong reason.

## What you should not do

- **Don't `std::env::set_var(...)` in a test.** It's `unsafe` in
  modern Rust and racy with parallel tests. The injection seam is
  the only safe way.
- **Don't mark the test `#[ignore]`.** That hides the problem
  instead of fixing it.
- **Don't make the assertion looser.** "`resp.source.starts_with('l')`"
  is not a contract — it's surrender. Tighten, don't loosen.

## How to detect more instances of this pattern

```bash
# All places that read env internally inside a fn that has tests.
rg -n 'std::env::var\("(\w*_API_KEY|\w*_TOKEN|\w*_SECRET|\w*_KEY)"\)' \
   --type rust

# All places that call select_healthy_chat_backend or similar.
rg -n 'select_healthy_chat_backend|ChatBackendSelectionConfig::from_env' \
   --type rust

# All tests that mention .envrc.
rg -n '\.envrc' tests/ --type rust
```

Each hit is a candidate for the same refactor.

## Track the work

Open a `QF-*` entry citing
[`RP-HERMETIC-UNIT`](../properties/RP-HERMETIC-UNIT.md) and a
specific test name. Reference this migration guide in the **Next
action** field. When the test is migrated, mark `Done` and link to
the merging commit.
