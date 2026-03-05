<script setup lang="ts">
import { computed, ref } from "vue";
import { useData } from "vitepress";
import HomeHeroShell from "./HomeHeroShell.vue";

const { lang } = useData();
const isZh = computed(() => lang.value === "zh-CN");

const quickStart = [
  "curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh",
  "eval \"$(envlock)\"",
  "echo \"$ENVLOCK_PROFILE\""
].join("\n");

type TierKey = "l4" | "l3" | "l2" | "l1";

type TierTool = {
  icon: string;
  name: string;
  url: string;
  tags: string[];
  abstract: string;
};

const activeTier = ref<TierKey>("l4");

const scoreTiers = computed(() => {
  if (isZh.value) {
    return [
      {
        key: "l4" as TierKey,
        level: "L4 native",
        rule: "AND：最强闭环 + 最小 Agent 成本同时成立。",
        tools: [
          {
            icon: "🐙",
            name: "gh",
            url: "https://cli.github.com/manual/",
            tags: ["cli", "native", "vcs"],
            abstract: "Agent 可直接闭环处理 issue/PR/review，步骤稳定且提示负担低。"
          },
          {
            icon: "☁️",
            name: "aws",
            url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html",
            tags: ["cloud", "native", "ops"],
            abstract: "服务 API 与命令语义一致，适合在约束清晰任务里快速闭环。"
          },
          {
            icon: "⎈",
            name: "kubectl",
            url: "https://kubernetes.io/docs/reference/kubectl/",
            tags: ["cluster", "native", "runtime"],
            abstract: "资源模型稳定、命令反馈结构化，便于 Agent 做排错与修复循环。"
          },
          {
            icon: "🧱",
            name: "tf",
            url: "https://developer.hashicorp.com/terraform/cli",
            tags: ["iac", "native", "plan/apply"],
            abstract: "plan/apply 工作流天然可验证，最适合 Agent 驱动基础设施闭环。"
          }
        ]
      },
      {
        key: "l3" as TierKey,
        level: "L3 good",
        rule: "OR：至少一条强闭环路径成熟，但覆盖未达 native。",
        tools: [
          {
            icon: "📈",
            name: "datadog",
            url: "https://docs.datadoghq.com/api/latest/",
            tags: ["observability", "good", "api"],
            abstract: "观测链路中有高质量闭环路径，但跨域任务时覆盖深度仍不足。"
          }
        ]
      },
      {
        key: "l2" as TierKey,
        level: "L2 normal",
        rule: "闭环存在，但通过 envlock 非兼容路径完成。",
        tools: [
          {
            icon: "⬢",
            name: "fnm",
            url: "https://github.com/Schniz/fnm",
            tags: ["runtime", "normal", "workaround"],
            abstract: "需要额外约定或外层封装才能与 envlock 的闭环语义对齐。"
          }
        ]
      },
      {
        key: "l1" as TierKey,
        level: "L1 other",
        rule: "无可靠闭环路径；在 Agent-Native 下是 non-sense。",
        tools: [
          {
            icon: "∅",
            name: "other",
            url: "/zh-CN/explanation/envlock-score/other",
            tags: ["other", "no-closure", "high-cost"],
            abstract: "缺乏可复制的闭环路径，不适合 Agent-first 的执行场景。"
          }
        ]
      }
    ];
  }

  return [
    {
      key: "l4" as TierKey,
      level: "L4 native",
      rule: "AND: strongest closure and minimum agent-side cost are both satisfied.",
      tools: [
        {
          icon: "🐙",
          name: "gh",
          url: "https://cli.github.com/manual/",
          tags: ["cli", "native", "vcs"],
          abstract: "Agents can close loops on issues, PRs, and reviews with stable low-cost prompts."
        },
        {
          icon: "☁️",
          name: "aws",
          url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html",
          tags: ["cloud", "native", "ops"],
          abstract: "Command semantics align with service APIs, enabling predictable constrained loops."
        },
        {
          icon: "⎈",
          name: "kubectl",
          url: "https://kubernetes.io/docs/reference/kubectl/",
          tags: ["cluster", "native", "runtime"],
          abstract: "Structured resource feedback supports fast diagnose-and-fix iteration by agents."
        },
        {
          icon: "🧱",
          name: "tf",
          url: "https://developer.hashicorp.com/terraform/cli",
          tags: ["iac", "native", "plan/apply"],
          abstract: "The plan/apply loop is verifiable by design and fits agent execution well."
        }
      ]
    },
    {
      key: "l3" as TierKey,
      level: "L3 good",
      rule: "OR: at least one strong closure path is mature, but coverage is below native.",
      tools: [
        {
          icon: "📈",
          name: "datadog",
          url: "https://docs.datadoghq.com/api/latest/",
          tags: ["observability", "good", "api"],
          abstract: "Strong closure exists in specific observability paths, but broad loop coverage is incomplete."
        }
      ]
    },
    {
      key: "l2" as TierKey,
      level: "L2 normal",
      rule: "Closure exists through envlock-non-compatible paths.",
      tools: [
        {
          icon: "⬢",
          name: "fnm",
          url: "https://github.com/Schniz/fnm",
          tags: ["runtime", "normal", "workaround"],
          abstract: "Closure is possible but needs wrappers or conventions outside envlock-compatible flow."
        }
      ]
    },
    {
      key: "l1" as TierKey,
      level: "L1 other",
      rule: "No reliable closure path; non-sense for Agent-Native workflows.",
      tools: [
        {
          icon: "∅",
          name: "other",
          url: "/explanation/envlock-score/other",
          tags: ["other", "no-closure", "high-cost"],
          abstract: "No reproducible closure path under unconstrained execution conditions."
        }
      ]
    }
  ];
});

const activeTierData = computed(() => scoreTiers.value.find((tier) => tier.key === activeTier.value) ?? scoreTiers.value[0]);

function hashTag(tag: string): number {
  let hash = 0;
  for (let i = 0; i < tag.length; i += 1) {
    hash = (hash << 5) - hash + tag.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

function tagStyle(tag: string): Record<string, string> {
  return {
    "--tag-h": `${hashTag(tag) % 360}`
  };
}

const docsLinks = computed(() => {
  if (isZh.value) {
    return [
      { label: "快速开始", link: "/zh-CN/tutorials/quick-start" },
      { label: "安装", link: "/zh-CN/how-to/install" },
      { label: "CLI 参考", link: "/zh-CN/reference/cli" },
      { label: "常见用法", link: "/zh-CN/how-to/common-recipes" },
      { label: "CI 集成", link: "/zh-CN/how-to/ci-integration" },
      { label: "GEO 指数", link: "/zh-CN/explanation/geo-index" },
      { label: "关于", link: "/zh-CN/explanation/why-envlock" }
    ];
  }

  return [
    { label: "Quick Start", link: "/tutorials/quick-start" },
    { label: "Install", link: "/how-to/install" },
    { label: "CLI Reference", link: "/reference/cli" },
    { label: "Common Recipes", link: "/how-to/common-recipes" },
    { label: "CI Integration", link: "/how-to/ci-integration" },
    { label: "GEO Index", link: "/explanation/geo-index" },
    { label: "About", link: "/explanation/why-envlock" }
  ];
});

const labels = computed(() =>
  isZh.value
    ? {
        boardKicker: "Agent-Native Ranking",
        boardRule: "Rule Model",
        boardRuleBody: "L4 = AND（最强闭环 + 最小 Agent 成本）；L3 = OR（至少一条强闭环路径成立）；L2 = 通过 envlock 非兼容路径闭环；L1 = 无可靠闭环路径。",
        tierTabs: "Tier Tabs",
        tierRule: "Tier Rule",
        tableTitle: "Title",
        tableTags: "Tags",
        tableAbstract: "Abstract",
        quickTitle: "快速启动",
        quickHint: "3 行命令，冷启动完成闭环验证",
        docsTitle: "核心文档入口",
        docsHint: "按任务直达，减少往返扫描"
      }
    : {
        boardKicker: "Agent-Native Ranking",
        boardRule: "Rule Model",
        boardRuleBody: "L4 = AND(strongest closure, minimum agent-side cost). L3 = OR(strong closure path exists). L2 = closure through envlock-non-compatible path. L1 = no reliable closure path.",
        tierTabs: "Tier Tabs",
        tierRule: "Tier Rule",
        tableTitle: "Title",
        tableTags: "Tags",
        tableAbstract: "Abstract",
        quickTitle: "Quick Start",
        quickHint: "Three lines to verify cold-start closure",
        docsTitle: "Core Docs",
        docsHint: "Task-routed entrypoints for faster scans"
      }
);
</script>

<template>
  <wc-layout-grid class="home-landing-grid" aria-label="envlock landing grid" columns="24" rows="minmax(0, 1fr) minmax(0, 2fr)" gap="14px">
    <wc-layout-panel class="landing-board">
    <article class="landing-card">
      <p class="landing-kicker">{{ labels.boardKicker }}</p>
      <h2 class="landing-score-title"><span>Scoreboard</span><small>by envlock</small></h2>
      <div class="score-global-rule">
        <p class="score-global-rule-head">{{ labels.boardRule }}</p>
        <p class="score-global-rule-body">{{ labels.boardRuleBody }}</p>
      </div>

      <div class="score-tabs" role="tablist" :aria-label="labels.tierTabs">
        <button
          v-for="tier in scoreTiers"
          :key="tier.key"
          class="score-tab"
          :class="{ 'is-active': activeTier === tier.key }"
          role="tab"
          :id="`score-tab-${tier.key}`"
          :aria-controls="`score-panel-${tier.key}`"
          :aria-selected="activeTier === tier.key"
          type="button"
          @click="activeTier = tier.key"
        >
          {{ tier.level }}
        </button>
      </div>

      <div class="score-pane" role="tabpanel" :id="`score-panel-${activeTierData.key}`" :aria-labelledby="`score-tab-${activeTierData.key}`">
        <p class="score-pane-rule"><strong>{{ labels.tierRule }}:</strong> {{ activeTierData.rule }}</p>
        <div class="score-table-wrap">
          <table class="score-table">
            <thead>
              <tr>
                <th scope="col">{{ labels.tableTitle }}</th>
                <th scope="col">{{ labels.tableTags }}</th>
                <th scope="col">{{ labels.tableAbstract }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="tool in activeTierData.tools" :key="tool.name">
                <td>
                  <div class="entry-title">
                    <a class="score-tool-link" :href="tool.url" target="_blank" rel="noreferrer">
                      <span class="score-tool-icon" aria-hidden="true">{{ tool.icon }}</span>
                      <span>{{ tool.name }}</span>
                    </a>
                  </div>
                </td>
                <td>
                  <div class="score-tags">
                    <span class="score-tag" v-for="tag in tool.tags" :key="tag" :style="tagStyle(tag)">{{ tag }}</span>
                  </div>
                </td>
                <td class="entry-abstract">{{ tool.abstract }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </article>
    </wc-layout-panel>

    <wc-layout-panel class="landing-quickstart">
      <HomeHeroShell :heading="labels.quickTitle" :hint="labels.quickHint" :code="quickStart" />
    </wc-layout-panel>

    <wc-layout-panel class="landing-docs">
    <article class="landing-card">
      <h2>{{ labels.docsTitle }}</h2>
      <p class="landing-hint">{{ labels.docsHint }}</p>
      <nav class="landing-docs-links">
        <a v-for="item in docsLinks" :key="item.link" :href="item.link">{{ item.label }}</a>
      </nav>
    </article>
    </wc-layout-panel>
  </wc-layout-grid>
</template>
