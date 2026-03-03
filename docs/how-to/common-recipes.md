# Common Recipes

Copy-paste snippets for v0.2.1.

## 1) Node + npm registry env

```bash
mkdir -p ./profiles
cat > ./profiles/node-registry.json <<'JSON'
{
  "injections": [
    {
      "type": "env",
      "vars": {
        "NODE_ENV": "development",
        "NPM_CONFIG_REGISTRY": "https://registry.npmjs.org/"
      }
    }
  ]
}
JSON

eval "$(envlock --profile ./profiles/node-registry.json)"
npm config get registry
```

## 2) Kube context vars

```bash
mkdir -p ./profiles
cat > ./profiles/kube-dev.json <<JSON
{
  "injections": [
    {
      "type": "env",
      "vars": {
        "KUBECONFIG": "${HOME}/.kube/config-dev",
        "KUBE_CONTEXT": "dev-cluster"
      }
    }
  ]
}
JSON

envlock --profile ./profiles/kube-dev.json -- bash -lc 'echo "$KUBECONFIG | $KUBE_CONTEXT"'
```

## 3) Child command mode one-off

```bash
mkdir -p ./profiles
cat > ./profiles/one-off.json <<'JSON'
{
  "injections": [
    {
      "type": "env",
      "vars": {
        "ENVLOCK_PROFILE": "one-off"
      }
    }
  ]
}
JSON

envlock --profile ./profiles/one-off.json -- bash -lc 'echo "$ENVLOCK_PROFILE"'
echo "${ENVLOCK_PROFILE:-unset in parent shell}"
```

## 4) Explicit profile override for project-local profile

```bash
mkdir -p ./profiles
cat > ./profiles/project.local.json <<'JSON'
{
  "injections": [
    {
      "type": "env",
      "vars": {
        "APP_ENV": "local",
        "API_BASE_URL": "http://localhost:3000"
      }
    }
  ]
}
JSON

envlock --profile ./profiles/project.local.json --output json
```

Use this when you keep a project-only profile in the repo and do not want to change your default profile.

## 5) Preview safety check before run

```bash
mkdir -p ./profiles
cat > ./profiles/release-check.json <<'JSON'
{
  "injections": [
    {
      "type": "env",
      "vars": {
        "RELEASE_CHANNEL": "staging"
      }
    }
  ]
}
JSON

envlock preview --profile ./profiles/release-check.json
envlock --profile ./profiles/release-check.json --output json
```

Use `preview` first to confirm profile metadata in read-only mode, then run the profile.
