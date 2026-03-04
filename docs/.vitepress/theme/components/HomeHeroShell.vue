<script setup lang="ts">
defineProps<{
  title: string;
  code: string;
}>();
</script>

<template>
  <div class="hero-shell-panel" role="region" :aria-label="title">
    <div class="hero-shell-head">
      <span class="hero-shell-dot hero-shell-dot-red" aria-hidden="true"></span>
      <span class="hero-shell-dot hero-shell-dot-yellow" aria-hidden="true"></span>
      <span class="hero-shell-dot hero-shell-dot-green" aria-hidden="true"></span>
      <span class="hero-shell-title">{{ title }}</span>
    </div>
    <pre class="hero-shell-code"><code>{{ code }}</code></pre>
  </div>
</template>
