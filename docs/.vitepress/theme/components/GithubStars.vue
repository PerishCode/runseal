<script setup lang="ts">
import { onMounted, ref } from "vue";

const stars = ref<string>("...");
const CACHE_KEY = "runseal_github_stars_cache_v1";
const CACHE_TTL_MS = 30 * 60 * 1000;

function formatStars(value: number): string {
  if (value >= 1000) {
    return `${(value / 1000).toFixed(1)}k`;
  }
  return `${value}`;
}

onMounted(async () => {
  const now = Date.now();

  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (raw) {
      const cached = JSON.parse(raw) as { value?: string; ts?: number };
      if (
        typeof cached.value === "string" &&
        typeof cached.ts === "number" &&
        now - cached.ts < CACHE_TTL_MS
      ) {
        stars.value = cached.value;
        return;
      }
    }
  } catch {
    // Ignore cache parsing failures and fetch fresh data.
  }

  try {
    const response = await fetch("https://api.github.com/repos/PerishCode/runseal", {
      headers: { Accept: "application/vnd.github+json" }
    });
    if (!response.ok) {
      stars.value = "Star";
      return;
    }
    const payload = (await response.json()) as { stargazers_count?: number };
    const count = payload.stargazers_count;
    stars.value = typeof count === "number" ? `Star ${formatStars(count)}` : "Star";
  } catch {
    stars.value = "Star";
  }

  try {
    localStorage.setItem(
      CACHE_KEY,
      JSON.stringify({
        value: stars.value,
        ts: now
      })
    );
  } catch {
    // Ignore storage failures.
  }
});
</script>

<template>
  <a
    class="github-stars-link"
    href="https://github.com/PerishCode/runseal/stargazers"
    target="_blank"
    rel="noreferrer"
    aria-label="GitHub stargazers"
  >
    {{ stars }}
  </a>
</template>
