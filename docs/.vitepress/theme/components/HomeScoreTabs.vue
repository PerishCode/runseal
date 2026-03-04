<script setup lang="ts">
import { computed, ref } from "vue";
import { useData } from "vitepress";

type TabKey = "native" | "good" | "normal" | "other";

const active = ref<TabKey>("native");
const { lang } = useData();

const isZh = computed(() => lang.value === "zh-CN");

const labels = computed(() =>
  isZh.value
    ? {
        title: "envlock-score",
        native: "L4 native",
        good: "L3 good",
        normal: "L2 normal",
        other: "L1 other",
        principle: "评分原则",
        tools: "代表工具",
        none: "无"
      }
    : {
        title: "envlock-score",
        native: "L4 native",
        good: "L3 good",
        normal: "L2 normal",
        other: "L1 other",
        principle: "Scoring Principle",
        tools: "Representative Tools",
        none: "none"
      }
);

const content = computed(() => {
  if (isZh.value) {
      return {
      native: {
        principle:
          "AND：最强闭环能力与最小 Agent 成本同时成立，支持最小提示与最少步骤的稳定闭环。",
        tools: [
          { name: "gh", url: "https://cli.github.com/manual/" },
          { name: "aws", url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html" },
          { name: "kubectl", url: "https://kubernetes.io/docs/reference/kubectl/" },
          { name: "tf", url: "https://developer.hashicorp.com/terraform/cli" }
        ]
      },
      good: {
        principle:
          "OR：至少一条高信号闭环路径成熟，Agent 可用 envlock 友好方式闭环，但能力覆盖未达 native。",
        tools: [
          { name: "datadog", url: "https://docs.datadoghq.com/api/latest/" }
        ]
      },
      normal: {
        principle: "闭环存在，但通过 envlock 非兼容路径完成；需要额外包装或临时约定。",
        tools: [{ name: "fnm", url: "https://github.com/Schniz/fnm" }]
      },
      other: {
        principle: "不存在可靠闭环路径。在 Agent-Native 视角下是 non-sense。",
        tools: [] as Array<{ name: string; url: string }>
      }
    };
  }

  return {
    native: {
      principle:
        "AND: strongest closure and minimal agent-side cost are both satisfied. Stable loop execution needs minimal prompts and steps.",
      tools: [
        { name: "gh", url: "https://cli.github.com/manual/" },
        { name: "aws", url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html" },
        { name: "kubectl", url: "https://kubernetes.io/docs/reference/kubectl/" },
        { name: "tf", url: "https://developer.hashicorp.com/terraform/cli" }
      ]
    },
    good: {
      principle:
        "OR: at least one high-signal closure path is mature. Agent loops are envlock-friendly, but coverage is below native.",
      tools: [
        { name: "datadog", url: "https://docs.datadoghq.com/api/latest/" }
      ]
    },
    normal: {
      principle:
        "Closure exists, but through envlock-non-compatible paths. Extra wrappers or ad-hoc conventions are required.",
      tools: [{ name: "fnm", url: "https://github.com/Schniz/fnm" }]
    },
    other: {
      principle: "No reliable closure path. From an Agent-Native perspective this is non-sense.",
      tools: [] as Array<{ name: string; url: string }>
    }
  };
});
</script>

<template>
  <div class="home-prehero">
    <div class="home-prehero-inner">
      <div class="hero-score-tabs" aria-label="envlock score tabs">
        <div class="hero-score-tabs-head">
          <strong>{{ labels.title }}</strong>
          <div class="hero-score-tab-list" role="tablist">
            <button class="hero-score-tab" :class="{ active: active === 'native' }" role="tab" type="button" @click="active = 'native'">
              {{ labels.native }}
            </button>
            <button class="hero-score-tab" :class="{ active: active === 'good' }" role="tab" type="button" @click="active = 'good'">
              {{ labels.good }}
            </button>
            <button class="hero-score-tab" :class="{ active: active === 'normal' }" role="tab" type="button" @click="active = 'normal'">
              {{ labels.normal }}
            </button>
            <button class="hero-score-tab" :class="{ active: active === 'other' }" role="tab" type="button" @click="active = 'other'">
              {{ labels.other }}
            </button>
          </div>
        </div>
        <p class="hero-score-principle">
          <span>{{ labels.principle }}:</span>
          {{ content[active].principle }}
        </p>
        <p class="hero-score-tools">
          <span>{{ labels.tools }}:</span>
          <template v-if="content[active].tools.length > 0">
            <a v-for="item in content[active].tools" :key="item.name" :href="item.url" target="_blank" rel="noreferrer">{{ item.name }}</a>
          </template>
          <em v-else>{{ labels.none }}</em>
        </p>
      </div>
    </div>
  </div>
</template>
