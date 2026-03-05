<script setup lang="ts">
import { computed } from "vue";
import { useData, withBase } from "vitepress";

const { lang } = useData();
const locale = computed(() => (lang.value === "zh-CN" ? "zh-CN" : "en-US"));
</script>

<template>
  <wc-home-landing :locale="locale" :base="withBase('/')" />
</template>
