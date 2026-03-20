const HOME_DATA = {
  "en-US": {
    quickHint: "Three lines to verify cold-start closure",
    docsHint: "Task-routed entrypoints for faster scans",
    boardRuleBody:
      "L4 = AND(strongest closure, minimum agent-side cost). L3 = OR(strong closure path exists). L2 = closure through runseal-non-compatible path. L1 = no reliable closure path.",
    docsTitle: "Core Docs",
    quickTitle: "Quick Start",
    boardKicker: "Agent-Native Ranking",
    boardRule: "Rule Model",
    tierTabs: "Tier Tabs",
    tierRule: "Tier Rule",
    tableTitle: "Title",
    tableTags: "Tags",
    tableAbstract: "Abstract",
      docsLinks: [
        { label: "Install", link: "/how-to/install" },
        { label: "Use Profiles", link: "/how-to/use-profiles" },
        { label: "GEO Index", link: "/explanation/geo-index" },
        { label: "L4 Native", link: "/explanation/runseal-score/native" }
      ],
    tiers: [
      {
        key: "l4",
        level: "L4 native",
        rule: "AND: strongest closure and minimum agent-side cost are both satisfied.",
        tools: [
          {
            icon: "gh",
            name: "gh",
            url: "https://cli.github.com/manual/",
            tags: ["cli", "native", "vcs"],
            abstract: "Agents can close loops on issues, PRs, and reviews with stable low-cost prompts."
          },
          {
            icon: "aws",
            name: "aws",
            url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html",
            tags: ["cloud", "native", "ops"],
            abstract: "Command semantics align with service APIs, enabling predictable constrained loops."
          },
          {
            icon: "k8s",
            name: "kubectl",
            url: "https://kubernetes.io/docs/reference/kubectl/",
            tags: ["cluster", "native", "runtime"],
            abstract: "Structured resource feedback supports fast diagnose-and-fix iteration by agents."
          },
          {
            icon: "tf",
            name: "tf",
            url: "https://developer.hashicorp.com/terraform/cli",
            tags: ["iac", "native", "plan/apply"],
            abstract: "The plan/apply loop is verifiable by design and fits agent execution well."
          }
        ]
      },
      {
        key: "l3",
        level: "L3 good",
        rule: "OR: at least one strong closure path is mature, but coverage is below native.",
        tools: [
          {
            icon: "obs",
            name: "datadog",
            url: "https://docs.datadoghq.com/api/latest/",
            tags: ["observability", "good", "api"],
            abstract: "Strong closure exists in specific observability paths, but broad loop coverage is incomplete."
          }
        ]
      },
      {
        key: "l2",
        level: "L2 normal",
        rule: "Closure exists through runseal-non-compatible paths.",
        tools: [
          {
            icon: "rt",
            name: "fnm",
            url: "https://github.com/Schniz/fnm",
            tags: ["runtime", "normal", "workaround"],
            abstract: "Closure is possible but needs wrappers or conventions outside runseal-compatible flow."
          }
        ]
      },
      {
        key: "l1",
        level: "L1 other",
        rule: "No reliable closure path; non-sense for Agent-Native workflows.",
        tools: [
          {
            icon: "na",
            name: "other",
            url: "/explanation/runseal-score/other",
            tags: ["other", "no-closure", "high-cost"],
            abstract: "No reproducible closure path under unconstrained execution conditions."
          }
        ]
      }
    ]
  },
  "zh-CN": {
    quickHint: "3 行命令，冷启动完成闭环验证",
    docsHint: "按任务直达，减少往返扫描",
    boardRuleBody:
      "L4 = AND（最强闭环 + 最小 Agent 成本）；L3 = OR（至少一条强闭环路径成立）；L2 = 通过 runseal 非兼容路径闭环；L1 = 无可靠闭环路径。",
    docsTitle: "核心文档入口",
    quickTitle: "快速启动",
    boardKicker: "Agent-Native Ranking",
    boardRule: "Rule Model",
    tierTabs: "Tier Tabs",
    tierRule: "Tier Rule",
    tableTitle: "Title",
    tableTags: "Tags",
    tableAbstract: "Abstract",
      docsLinks: [
        { label: "安装", link: "/zh-CN/how-to/install" },
        { label: "使用 Profiles", link: "/zh-CN/how-to/use-profiles" },
        { label: "GEO 指数", link: "/zh-CN/explanation/geo-index" },
        { label: "L4 Native", link: "/zh-CN/explanation/runseal-score/native" }
      ],
    tiers: [
      {
        key: "l4",
        level: "L4 native",
        rule: "AND：最强闭环 + 最小 Agent 成本同时成立。",
        tools: [
          {
            icon: "gh",
            name: "gh",
            url: "https://cli.github.com/manual/",
            tags: ["cli", "native", "vcs"],
            abstract: "Agent 可直接闭环处理 issue/PR/review，步骤稳定且提示负担低。"
          },
          {
            icon: "aws",
            name: "aws",
            url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html",
            tags: ["cloud", "native", "ops"],
            abstract: "服务 API 与命令语义一致，适合在约束清晰任务里快速闭环。"
          },
          {
            icon: "k8s",
            name: "kubectl",
            url: "https://kubernetes.io/docs/reference/kubectl/",
            tags: ["cluster", "native", "runtime"],
            abstract: "资源模型稳定、命令反馈结构化，便于 Agent 做排错与修复循环。"
          },
          {
            icon: "tf",
            name: "tf",
            url: "https://developer.hashicorp.com/terraform/cli",
            tags: ["iac", "native", "plan/apply"],
            abstract: "plan/apply 工作流天然可验证，最适合 Agent 驱动基础设施闭环。"
          }
        ]
      },
      {
        key: "l3",
        level: "L3 good",
        rule: "OR：至少一条强闭环路径成熟，但覆盖未达 native。",
        tools: [
          {
            icon: "obs",
            name: "datadog",
            url: "https://docs.datadoghq.com/api/latest/",
            tags: ["observability", "good", "api"],
            abstract: "观测链路中有高质量闭环路径，但跨域任务时覆盖深度仍不足。"
          }
        ]
      },
      {
        key: "l2",
        level: "L2 normal",
        rule: "闭环存在，但通过 runseal 非兼容路径完成。",
        tools: [
          {
            icon: "rt",
            name: "fnm",
            url: "https://github.com/Schniz/fnm",
            tags: ["runtime", "normal", "workaround"],
            abstract: "需要额外约定或外层封装才能与 runseal 的闭环语义对齐。"
          }
        ]
      },
      {
        key: "l1",
        level: "L1 other",
        rule: "无可靠闭环路径；在 Agent-Native 下是 non-sense。",
        tools: [
          {
            icon: "na",
            name: "other",
            url: "/zh-CN/explanation/runseal-score/other",
            tags: ["other", "no-closure", "high-cost"],
            abstract: "缺乏可复制的闭环路径，不适合 Agent-first 的执行场景。"
          }
        ]
      }
    ]
  }
};

const QUICK_START = [
  "curl -fsSL https://raw.githubusercontent.com/PerishCode/runseal/main/scripts/manage/install.sh | sh",
  "eval \"$(runseal)\"",
  "echo \"$RUNSEAL_PROFILE\""
].join("\n");

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function hashTag(tag) {
  let hash = 0;
  for (let i = 0; i < tag.length; i += 1) {
    hash = (hash << 5) - hash + tag.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

function isExternal(url) {
  return url.startsWith("http://") || url.startsWith("https://");
}

function resolveHref(url, base) {
  if (isExternal(url) || url.startsWith("#")) {
    return url;
  }

  const safeBase = base.endsWith("/") ? base : `${base}/`;
  const safePath = url.startsWith("/") ? url.slice(1) : url;
  return `${safeBase}${safePath}`;
}

class WcHomeLanding extends HTMLElement {
  static get observedAttributes() {
    return ["locale", "base", "active-tier"];
  }

  connectedCallback() {
    if (!this.shadowRoot) {
      this.attachShadow({ mode: "open" });
    }
    this.render();
  }

  attributeChangedCallback() {
    if (this.isConnected) {
      this.render();
    }
  }

  getData() {
    const locale = this.getAttribute("locale") === "zh-CN" ? "zh-CN" : "en-US";
    return HOME_DATA[locale];
  }

  getBase() {
    const base = this.getAttribute("base") || "/";
    return base.endsWith("/") ? base : `${base}/`;
  }

  getActiveTier(data) {
    const tier = this.getAttribute("active-tier") || "l4";
    const exists = data.tiers.some((item) => item.key === tier);
    return exists ? tier : "l4";
  }

  render() {
    const data = this.getData();
    const activeTier = this.getActiveTier(data);
    const base = this.getBase();

    const tabs = data.tiers
      .map(
        (tier) => `<button
          class="score-tab ${tier.key === activeTier ? "is-active" : ""}"
          role="tab"
          id="score-tab-${tier.key}"
          aria-controls="score-panel-${tier.key}"
          aria-selected="${tier.key === activeTier ? "true" : "false"}"
          tabindex="${tier.key === activeTier ? "0" : "-1"}"
          type="button"
          data-tier="${tier.key}">
          ${escapeHtml(tier.level)}
        </button>`
      )
      .join("");

    const renderRows = (tools) =>
      tools
        .map((tool) => {
          const tags = tool.tags
            .map((tag) => {
              const hue = hashTag(tag) % 360;
              return `<span class="score-tag" style="--tag-h:${hue}">${escapeHtml(tag)}</span>`;
            })
            .join("");

          const href = resolveHref(tool.url, base);
          const externalAttrs = isExternal(tool.url) ? ' target="_blank" rel="noreferrer"' : "";

          return `<tr>
            <td>
              <div class="entry-title">
                <a class="score-tool-link" href="${escapeHtml(href)}"${externalAttrs}>
                  <span class="score-tool-icon" aria-hidden="true">${escapeHtml(tool.icon)}</span>
                  <span>${escapeHtml(tool.name)}</span>
                </a>
              </div>
            </td>
            <td>
              <div class="score-tags">${tags}</div>
            </td>
            <td class="entry-abstract">${escapeHtml(tool.abstract)}</td>
          </tr>`;
        })
        .join("");

    const panels = data.tiers
      .map(
        (tier) => `<div class="score-pane" role="tabpanel" id="score-panel-${tier.key}" aria-labelledby="score-tab-${tier.key}" ${
          tier.key === activeTier ? "" : "hidden"
        }>
            <p class="score-pane-rule"><strong>${escapeHtml(data.tierRule)}:</strong> ${escapeHtml(tier.rule)}</p>
            <div class="score-table-wrap">
              <table class="score-table">
                <thead>
                  <tr>
                    <th scope="col">${escapeHtml(data.tableTitle)}</th>
                    <th scope="col">${escapeHtml(data.tableTags)}</th>
                    <th scope="col">${escapeHtml(data.tableAbstract)}</th>
                  </tr>
                </thead>
                <tbody>${renderRows(tier.tools)}</tbody>
              </table>
            </div>
          </div>`
      )
      .join("");

    const links = data.docsLinks
      .map((item) => `<a href="${escapeHtml(resolveHref(item.link, base))}">${escapeHtml(item.label)}</a>`)
      .join("");

    this.shadowRoot.innerHTML = `<link rel="stylesheet" href="${base}home-layout-wc.css">
      <section class="home-landing-grid" aria-label="runseal landing grid">
        <article class="landing-card landing-board">
          <p class="landing-kicker">${escapeHtml(data.boardKicker)}</p>
          <h2 class="landing-score-title"><span>Scoreboard</span><small>by runseal</small></h2>
          <div class="score-global-rule">
            <p class="score-global-rule-head">${escapeHtml(data.boardRule)}</p>
            <p class="score-global-rule-body">${escapeHtml(data.boardRuleBody)}</p>
          </div>
          <div class="score-tabs" role="tablist" aria-label="${escapeHtml(data.tierTabs)}">${tabs}</div>
          ${panels}
        </article>

        <article class="landing-card landing-quickstart" role="region" aria-label="${escapeHtml(data.quickTitle)}">
          <div class="hero-shell-head">
            <span class="hero-shell-dot hero-shell-dot-red" aria-hidden="true"></span>
            <span class="hero-shell-dot hero-shell-dot-yellow" aria-hidden="true"></span>
            <span class="hero-shell-dot hero-shell-dot-green" aria-hidden="true"></span>
            <span class="hero-shell-title">${escapeHtml(data.quickTitle)}</span>
          </div>
          <p class="hero-shell-hint">${escapeHtml(data.quickHint)}</p>
          <pre class="hero-shell-code"><code>${escapeHtml(QUICK_START)}</code></pre>
        </article>

        <article class="landing-card landing-docs">
          <h2>${escapeHtml(data.docsTitle)}</h2>
          <p class="landing-hint">${escapeHtml(data.docsHint)}</p>
          <nav class="landing-docs-links">${links}</nav>
        </article>
      </section>`;

    this.shadowRoot.querySelectorAll("[data-tier]").forEach((button) => {
      button.addEventListener("click", () => {
        const tier = button.getAttribute("data-tier");
        if (tier) {
          this.setAttribute("active-tier", tier);
        }
      });

      button.addEventListener("keydown", (event) => {
        const tabsList = Array.from(this.shadowRoot.querySelectorAll("[data-tier]"));
        const index = tabsList.indexOf(button);
        if (index < 0) {
          return;
        }

        if (event.key === "ArrowRight") {
          event.preventDefault();
          const next = tabsList[(index + 1) % tabsList.length];
          const tier = next.getAttribute("data-tier");
          if (tier) {
            this.setAttribute("active-tier", tier);
          }
        }

        if (event.key === "ArrowLeft") {
          event.preventDefault();
          const prev = tabsList[(index - 1 + tabsList.length) % tabsList.length];
          const tier = prev.getAttribute("data-tier");
          if (tier) {
            this.setAttribute("active-tier", tier);
          }
        }
      });
    });
  }
}

if (!customElements.get("wc-home-landing")) {
  customElements.define("wc-home-landing", WcHomeLanding);
}
