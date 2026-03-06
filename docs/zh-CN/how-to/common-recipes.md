# 常见用法

面向 v0.4.2 的可直接复制命令。

## 1) Node + npm registry 环境

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

## 2) Kube context 变量

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

## 3) 子命令模式（一次性）

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

## 4) 显式覆盖为项目内 profile

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

适合仓库内维护项目专属 profile，同时不影响默认 profile。

## 5) 运行前做 preview 安全检查

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

先 `preview`（只读）确认元信息，再执行正式运行。
