import DefaultTheme from "vitepress/theme";
import { h } from "vue";
import GithubStars from "./components/GithubStars.vue";
import HomeHeroShell from "./components/HomeHeroShell.vue";
import HomeScoreTabs from "./components/HomeScoreTabs.vue";
import "./style.css";

export default {
  extends: DefaultTheme,
  Layout() {
    return h(DefaultTheme.Layout, null, {
      "nav-bar-content-after": () => h(GithubStars),
      "home-hero-info-before": () => h(HomeScoreTabs),
      "home-hero-image": () => h(HomeHeroShell)
    });
  }
};
