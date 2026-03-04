<script setup lang="ts">
import { computed, ref } from "vue";
import { useData } from "vitepress";

type TabKey = "good" | "normal" | "other";

const active = ref<TabKey>("good");
const { lang } = useData();

const isZh = computed(() => lang.value === "zh-CN");

const labels = computed(() =>
  isZh.value
    ? {
        title: "envlock-score",
        good: "good",
        normal: "normal",
        other: "other",
        principle: "评分原则",
        tools: "代表工具",
        none: "无"
      }
    : {
        title: "envlock-score",
        good: "good",
        normal: "normal",
        other: "other",
        principle: "Scoring Principle",
        tools: "Representative Tools",
        none: "none"
      }
);

const content = computed(() => {
  if (isZh.value) {
    return {
      good: {
        principle:
          "具备成熟编排闭环；当前以 env+symlink 闭环为高信号，长期向全能力开放编排收敛。",
        tools: [
          { name: "gh", url: "https://cli.github.com/manual/" },
          { name: "aws", url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html" },
          { name: "kubectl", url: "https://kubernetes.io/docs/reference/kubectl/" },
          { name: "datadog api", url: "https://docs.datadoghq.com/api/latest/" }
        ]
      },
      normal: {
        principle: "至少具备 command 闭环，可被 Agent 稳定调用，但长期编排能力弱于 good。",
        tools: [{ name: "fnm", url: "https://github.com/Schniz/fnm" }]
      },
      other: {
        principle: "不具备 command 闭环；在 Agent-Native 视角下是 non-sense。",
        tools: [] as Array<{ name: string; url: string }>
      }
    };
  }

  return {
    good: {
      principle:
        "Mature orchestration closure. Today, env+symlink closure is a high-signal marker; long term it converges toward open orchestration for all capabilities.",
      tools: [
        { name: "gh", url: "https://cli.github.com/manual/" },
        { name: "aws", url: "https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html" },
        { name: "kubectl", url: "https://kubernetes.io/docs/reference/kubectl/" },
        { name: "datadog api", url: "https://docs.datadoghq.com/api/latest/" }
      ]
    },
    normal: {
      principle:
        "At least command-closure. Agent usage is stable, but long-horizon orchestration value is weaker than good.",
      tools: [{ name: "fnm", url: "https://github.com/Schniz/fnm" }]
    },
    other: {
      principle: "No command-closure. From an Agent-Native perspective this is non-sense.",
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
