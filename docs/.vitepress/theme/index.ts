import DefaultTheme from "vitepress/theme";
import { h } from "vue";
import GithubStars from "./components/GithubStars.vue";
import HomeLandingHost from "./components/HomeLandingHost.vue";
import "./style.css";

export default {
  extends: DefaultTheme,
  Layout() {
    return h(DefaultTheme.Layout, null, {
      "nav-bar-content-after": () => h(GithubStars),
      "home-hero-before": () => h(HomeLandingHost)
    });
  }
};
