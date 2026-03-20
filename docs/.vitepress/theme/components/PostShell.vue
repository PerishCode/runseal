<script setup lang="ts">
import { computed, onBeforeUnmount, watchEffect } from "vue";
import { useData } from "vitepress";

const { frontmatter } = useData();
const isPost = computed(() => frontmatter.value?.bodyClass === "post-page");
const title = computed(() => frontmatter.value?.title ?? "");
const meta = computed(() => frontmatter.value?.meta ?? "");

watchEffect(() => {
  if (typeof document === "undefined") return;
  document.body.classList.toggle("post-page", isPost.value);
});

onBeforeUnmount(() => {
  if (typeof document === "undefined") return;
  document.body.classList.remove("post-page");
});
</script>

<template>
  <div v-if="isPost" class="post-shell">
    <div class="post-shell__card">
      <h1 class="post-shell__title">{{ title }}</h1>
      <div v-if="meta" class="post-shell__meta">{{ meta }}</div>
    </div>
  </div>
</template>
