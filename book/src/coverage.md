# Coverage

Every v1 method name, on both sides. Beyond the stable core, all eight areas the
spec still marks unstable are implemented: session fork, LLM providers, plan
operations, next edit suggestions, end-of-turn token usage, tool call names,
auth methods, and MCP over ACP.

## Extension

Both mechanisms the spec defines work:

- `_meta` is a field on every message that carries it in the spec, read and
  written untouched
- `_`-prefixed methods reach `ext_request` / `ext_notification` on either role,
  which decline by default like every other optional method

## Unknown shapes

Update kinds, content blocks, tool call content, plan payloads and enum values
this revision does not know arrive in an `Other` variant and round-trip whole,
rather than failing the message they came in. A newer peer does not break you.

## Not there yet

The leniency upstream applies field by field, where a malformed optional field
falls back to its default and a bad array item is skipped rather than failing
the message around it. Unknown *shapes* are handled — that is what the `Other`
variants are for — but malformed ones are still an error.
