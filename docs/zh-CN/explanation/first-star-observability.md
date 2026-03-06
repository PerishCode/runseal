# 0 社媒首星观测

本页定义低样本阶段（0 社媒）的最小观测框架，用于判断离首个自然 GitHub star 还有多远。

## 漏斗

- 曝光：仓库浏览量（`views.count`，近 7 天）
- 到达：访客代理（`views.uniques`，近 7 天）
- 理解：阅读深度代理（`views.count / max(views.uniques, 1)`）
- 尝试：克隆代理（`clones.uniques`，近 7 天）
- 认可：近 7 天新增 star（`stars_new_7d`）
- star：是否达成首星（`stargazers_count >= 1`）

## 每周评分（0-100）

- 曝光：0-20
- 到达：0-20
- 理解：0-20
- 尝试：0-20
- 认可：0-10
- star：0-10

公式：

- `exposure = min(20, views_7d / 30 * 20)`
- `reach = min(20, visitors_7d / 5 * 20)`
- `understand = min(20, max(0, (depth - 1.0) / 0.8 * 20))`
- `try = min(20, clones_uniques_7d / 4 * 20)`
- `approve = min(10, stars_new_7d * 10)`
- `star = 10 if stargazers_count >= 1 else 0`

当 traffic 数据不可用时，按可用维度归一化总分。

## 执行

```bash
python3 scripts/observe_first_star.py --repo PerishCode/envlock --days 7
```

推荐加 token：

```bash
GITHUB_TOKEN=xxx python3 scripts/observe_first_star.py --repo PerishCode/envlock --days 7
```

在 GitHub Actions 中，建议配置仓库 Secret `FIRST_STAR_GH_TOKEN`（具备 repo traffic 读取权限），否则 traffic 接口可能返回 `403`。

JSON 输出：

```bash
python3 scripts/observe_first_star.py --repo PerishCode/envlock --days 7 --json
```

## 关键转化持久化

- 工作流：`First Star Observer`
- 当检测到 `stars_total >= 1` 时，工作流会自动创建前缀为 `[CVR][first_star_reached]` 的 GitHub issue。
- 该 issue 作为关键转化瞬间的可追溯时间记录。
- 每次观测运行还会向月度总账 issue（`[CVR Ledger] YYYY-MM envlock conversion log`）追加快照评论。
