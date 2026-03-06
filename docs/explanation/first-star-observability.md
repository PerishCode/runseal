# First-Star Observability (0 Social)

This page defines a low-sample observability baseline for the first natural GitHub star.

Question to answer every week:

- How close are we to the first star without social distribution?

## Funnel

- Exposure: repo views (`views.count`, last 7 days)
- Reach: visitor proxy (`views.uniques`, last 7 days)
- Understand: depth proxy (`views.count / max(views.uniques, 1)`)
- Try: clone proxy (`clones.uniques`, last 7 days)
- Approve: new stars in window (`stars_new_7d`)
- Star: first star reached (`stargazers_count >= 1`)

## Weekly Score (0-100)

- Exposure: 0-20
- Reach: 0-20
- Understand: 0-20
- Try: 0-20
- Approve: 0-10
- Star: 0-10

Formulas:

- `exposure = min(20, views_7d / 30 * 20)`
- `reach = min(20, visitors_7d / 5 * 20)`
- `understand = min(20, max(0, (depth - 1.0) / 0.8 * 20))`
- `try = min(20, clones_uniques_7d / 4 * 20)`
- `approve = min(10, stars_new_7d * 10)`
- `star = 10 if stargazers_count >= 1 else 0`

When traffic data is unavailable, normalize by available dimensions.

## Run

```bash
python3 scripts/observe_first_star.py --repo PerishCode/envlock --days 7
```

With token (recommended):

```bash
GITHUB_TOKEN=xxx python3 scripts/observe_first_star.py --repo PerishCode/envlock --days 7
```

For GitHub Actions, set repository secret `FIRST_STAR_GH_TOKEN` (PAT with repo traffic access) to avoid `403` on traffic endpoints.

JSON output:

```bash
python3 scripts/observe_first_star.py --repo PerishCode/envlock --days 7 --json
```
