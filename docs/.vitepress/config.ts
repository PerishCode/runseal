import { defineConfig } from "vitepress";

export default defineConfig({
  title: "envlock",
  description: "Deterministic environment sessions from JSON profiles.",
  base: "/envlock/",
  cleanUrls: true,
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: "EN", link: "/" },
      { text: "中文", link: "/zh-CN/" },
      { text: "Tutorial", link: "/tutorials/quick-start" },
      { text: "How-to", link: "/how-to/install" },
      { text: "Reference", link: "/reference/cli" },
      { text: "Explanation", link: "/explanation/design-boundaries" },
      { text: "GitHub", link: "https://github.com/PerishCode/envlock" }
    ],
    sidebar: {
      "/": [
        {
          text: "Tutorial",
          items: [{ text: "Quick Start", link: "/tutorials/quick-start" }]
        },
        {
          text: "How-to",
          items: [
            { text: "Install", link: "/how-to/install" },
            { text: "Common Recipes", link: "/how-to/common-recipes" },
            { text: "Migrate to v0.2", link: "/how-to/migrate-to-v0.2" },
            { text: "Use Profiles", link: "/how-to/use-profiles" },
            { text: "Run Command Mode", link: "/how-to/command-mode" },
             { text: "CI Integration", link: "/how-to/ci-integration" },
             { text: "Release Validation", link: "/how-to/release-validation" },
             { text: "Release Operator Playbook", link: "/how-to/release-operator-playbook" },
             { text: "Update and Uninstall", link: "/how-to/update-and-uninstall" }
           ]
         },
        {
          text: "Reference",
          items: [
            { text: "Quick Reference", link: "/reference/quick-reference" },
            { text: "CLI", link: "/reference/cli" },
            { text: "Profile Format", link: "/reference/profile" },
            { text: "Environment Variables", link: "/reference/environment" },
            { text: "Release Pipeline", link: "/reference/release" }
          ]
        },
        {
          text: "Explanation",
          items: [
            { text: "Why envlock", link: "/explanation/why-envlock" },
            { text: "FAQ", link: "/explanation/faq" },
            { text: "Design Boundaries", link: "/explanation/design-boundaries" },
            { text: "Troubleshooting", link: "/explanation/troubleshooting" },
            { text: "Support Policy", link: "/explanation/support-policy" },
            { text: "Language Maintenance", link: "/explanation/language-maintenance" }
          ]
        }
      ],
      "/zh-CN/": [
        {
          text: "中文文档",
          items: [
            { text: "首页", link: "/zh-CN/" },
            { text: "安装", link: "/zh-CN/how-to/install" },
             { text: "常用配方", link: "/zh-CN/how-to/common-recipes" },
             { text: "CI 集成", link: "/zh-CN/how-to/ci-integration" },
             { text: "发布验证", link: "/zh-CN/how-to/release-validation" },
             { text: "发布操作手册", link: "/zh-CN/how-to/release-operator-playbook" },
             { text: "快速参考", link: "/zh-CN/reference/quick-reference" },
             { text: "CLI 参考", link: "/zh-CN/reference/cli" },
            { text: "迁移到 v0.2", link: "/zh-CN/how-to/migrate-to-v0.2" },
            { text: "常见问题", link: "/zh-CN/explanation/faq" }
          ]
        }
      ]
    },
    outline: {
      level: [2, 3],
      label: "On this page"
    },
    search: {
      provider: "local"
    },
    editLink: {
      pattern: "https://github.com/PerishCode/envlock/edit/main/docs/:path",
      text: "Edit this page on GitHub"
    },
    socialLinks: [{ icon: "github", link: "https://github.com/PerishCode/envlock" }],
    footer: {
      message: "Built with VitePress",
      copyright: "Copyright © 2026 PerishCode"
    }
  }
});
