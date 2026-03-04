<script setup lang="ts">
import { computed } from "vue";
import { useData } from "vitepress";

const { lang } = useData();
const isZh = computed(() => lang.value === "zh-CN");

const quickStart = [
  "curl -fsSL https://raw.githubusercontent.com/PerishCode/envlock/main/scripts/install.sh | sh",
  "eval \"$(envlock)\"",
  "echo \"$ENVLOCK_PROFILE\""
].join("\n");

const scoreRows = computed(() => {
  if (isZh.value) {
    return [
      {
        level: "L4 native",
        rule: "AND：最强闭环 + 最小 Agent 成本同时成立。",
        tools: [
          { name: "gh", url: "https://cli.github.com/manual/" },
          { name: "aws", url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html" },
          { name: "kubectl", url: "https://kubernetes.io/docs/reference/kubectl/" },
          { name: "tf", url: "https://developer.hashicorp.com/terraform/cli" }
        ]
      },
      {
        level: "L3 good",
        rule: "OR：至少一条强闭环路径成熟，但覆盖未达 native。",
        tools: [{ name: "datadog", url: "https://docs.datadoghq.com/api/latest/" }]
      },
      {
        level: "L2 normal",
        rule: "闭环存在，但通过 envlock 非兼容路径完成。",
        tools: [{ name: "fnm", url: "https://github.com/Schniz/fnm" }]
      },
      {
        level: "L1 other",
        rule: "无可靠闭环路径；在 Agent-Native 下是 non-sense。",
        tools: [] as Array<{ name: string; url: string }>
      }
    ];
  }

  return [
    {
      level: "L4 native",
      rule: "AND: strongest closure and minimum agent-side cost are both satisfied.",
      tools: [
        { name: "gh", url: "https://cli.github.com/manual/" },
        { name: "aws", url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html" },
        { name: "kubectl", url: "https://kubernetes.io/docs/reference/kubectl/" },
        { name: "tf", url: "https://developer.hashicorp.com/terraform/cli" }
      ]
    },
    {
      level: "L3 good",
      rule: "OR: at least one strong closure path is mature, but coverage is below native.",
      tools: [{ name: "datadog", url: "https://docs.datadoghq.com/api/latest/" }]
    },
    {
      level: "L2 normal",
      rule: "Closure exists through envlock-non-compatible paths.",
      tools: [{ name: "fnm", url: "https://github.com/Schniz/fnm" }]
    },
    {
      level: "L1 other",
      rule: "No reliable closure path; non-sense for Agent-Native workflows.",
      tools: [] as Array<{ name: string; url: string }>
    }
  ];
});

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
        quickTitle: "快速启动",
        docsTitle: "核心文档入口",
        scoreRule: "评分原则",
        tools: "工具",
        none: "无"
      }
    : {
        quickTitle: "Quick Start",
        docsTitle: "Core Docs",
        scoreRule: "Rule",
        tools: "Tools",
        none: "none"
      }
);
</script>

<template>
  <section class="home-landing-shell" aria-label="envlock landing shell">
    <section class="home-landing-grid" aria-label="envlock landing grid">
      <article class="landing-card landing-board">
        <h2 class="landing-score-title"><span>Scoreboard</span><small>by envlock</small></h2>
        <div class="score-accordion">
          <details class="score-item" v-for="row in scoreRows" :key="row.level" :open="row.level.startsWith('L4')">
            <summary class="score-item-head">{{ row.level }}</summary>
            <div class="score-item-body">
              <p><strong>{{ labels.scoreRule }}:</strong> {{ row.rule }}</p>
              <p>
                <strong>{{ labels.tools }}:</strong>
                <template v-if="row.tools.length > 0">
                  <a v-for="tool in row.tools" :key="tool.name" :href="tool.url" target="_blank" rel="noreferrer">{{ tool.name }}</a>
                </template>
                <em v-else>{{ labels.none }}</em>
              </p>
            </div>
          </details>
        </div>
      </article>

      <article class="landing-card landing-quickstart">
        <h2>{{ labels.quickTitle }}</h2>
        <pre><code>{{ quickStart }}</code></pre>
      </article>

      <article class="landing-card landing-docs">
        <h2>{{ labels.docsTitle }}</h2>
        <nav class="landing-docs-links">
          <a v-for="item in docsLinks" :key="item.link" :href="item.link">{{ item.label }}</a>
        </nav>
      </article>
    </section>
  </section>
</template>
