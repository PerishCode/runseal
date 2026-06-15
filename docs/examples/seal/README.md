# Seal Target Syntax Examples

These examples describe the intended isolated Seal source syntax. They are
design targets for discussion, not a claim that the current runtime already
parses every shape.

Seal models finite operational flow:

- Seal method calls for repo-owned operations.
- External process nodes for ordinary developer infrastructure.
- `@` tool calls for structured runseal capabilities.
- Explicit variables, environment access, control flow, IO, and stream flow.

The author-facing syntax should not preserve bash surface forms for familiarity.
Cross-platform behavior belongs in the runseal runtime, not in shell-shaped
source code.

## Files

- [Grammar draft](./grammar.md)
- [Terminology and lowering model](./semantics.md)
- [Perish workflow sketches](./perish-scenarios.md)
- [Calls](./calls.md)
- [Control flow](./control-flow.md)
- [Environment and scope](./env-scope.md)
- [IO and pipelines](./io-pipeline.md)
- [Values and collections](./values.md)
- [Full stream model sketch](./stream-model.md)
- [Full failure model sketch](./failure-model.md)
- [Argv parsing](./case.md)
