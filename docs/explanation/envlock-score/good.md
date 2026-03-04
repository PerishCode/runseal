# envlock-score/good

Rule (hard): has mature orchestration closure for runtime control.

- `env` closure: runtime behavior can be fully controlled by environment variables.
- `symlink` closure: runtime entrypoint/context can be switched by stable symlink routing.
- This tier accepts closure-first maturity now and evolves toward `native` over time.

In Agent-Native workflows, this is the target baseline.

Representative cases:

- [GitHub CLI](https://cli.github.com/manual/)
- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)
- [kubectl](https://kubernetes.io/docs/reference/kubectl/)
