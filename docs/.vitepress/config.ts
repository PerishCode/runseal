import { defineConfig } from "vitepress";

const BASE = "/envlock/";

export default defineConfig({
  title: "envlock",
  description: "Deterministic environment sessions from JSON profiles.",
  base: BASE,
  head: [["link", { rel: "icon", type: "image/png", href: `${BASE}favicon.png` }]],
  cleanUrls: true,
  lastUpdated: true,
  locales: {
    root: {
      lang: "en-US",
      label: "English",
      link: "/",
      title: "envlock",
      description: "Deterministic environment sessions from JSON profiles.",
      themeConfig: {
        nav: [
          { text: "Tutorial", link: "/tutorials/quick-start" },
          { text: "How-to", link: "/how-to/install" },
          { text: "Reference", link: "/reference/cli" },
          { text: "Explanation", link: "/explanation/design-boundaries" },
          { text: "GitHub", link: "https://github.com/PerishCode/envlock" }
        ],
        outline: {
          level: [2, 3],
          label: "On this page"
        },
        editLink: {
          pattern: "https://github.com/PerishCode/envlock/edit/main/docs/:path",
          text: "Edit this page on GitHub"
        },
        localeLinks: {
          text: "Language"
        }
      }
    },
    "zh-CN": {
      lang: "zh-CN",
      label: "简体中文",
      link: "/zh-CN/",
      title: "envlock",
      description: "通过 JSON 配置实现可复现环境会话。",
      themeConfig: {
        nav: [
          { text: "教程", link: "/tutorials/quick-start" },
          { text: "操作指南", link: "/zh-CN/how-to/install" },
          { text: "参考", link: "/zh-CN/reference/cli" },
          { text: "说明", link: "/zh-CN/explanation/faq" },
          { text: "GitHub", link: "https://github.com/PerishCode/envlock" }
        ],
        outline: {
          level: [2, 3],
          label: "本页导航"
        },
        editLink: {
          pattern: "https://github.com/PerishCode/envlock/edit/main/docs/:path",
          text: "在 GitHub 上编辑此页"
        },
        localeLinks: {
          text: "语言"
        }
      }
    }
  },
  themeConfig: {
    i18nRouting: true,
    logo: "/favicon.png",
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
            { text: "Migrate to v0.3", link: "/how-to/migrate-to-v0.3" },
            { text: "Use Profiles", link: "/how-to/use-profiles" },
            { text: "Run Command Mode", link: "/how-to/command-mode" },
            { text: "CI Integration", link: "/how-to/ci-integration" },
            { text: "Release Validation", link: "/how-to/release-validation" },
            { text: "Release Operator Playbook", link: "/how-to/release-operator-playbook" },
            { text: "Update and Uninstall", link: "/how-to/update-and-uninstall" },
            { text: "Docs Maintenance", link: "/how-to/docs-maintenance" }
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
          text: "教程",
          items: [{ text: "快速开始（英文）", link: "/tutorials/quick-start" }]
        },
        {
          text: "操作指南",
          items: [
            { text: "安装", link: "/zh-CN/how-to/install" },
            { text: "常见用法", link: "/zh-CN/how-to/common-recipes" },
            { text: "迁移到 v0.3", link: "/zh-CN/how-to/migrate-to-v0.3" },
            { text: "使用 Profiles（英文）", link: "/how-to/use-profiles" },
            { text: "子命令模式", link: "/zh-CN/how-to/command-mode" },
            { text: "CI 集成", link: "/zh-CN/how-to/ci-integration" },
            { text: "发布验证", link: "/zh-CN/how-to/release-validation" },
            { text: "发布操作指南", link: "/zh-CN/how-to/release-operator-playbook" },
            { text: "更新与卸载（英文）", link: "/how-to/update-and-uninstall" },
            { text: "文档维护", link: "/zh-CN/how-to/docs-maintenance" },
          ]
        },
        {
          text: "参考",
          items: [
            { text: "快速参考", link: "/zh-CN/reference/quick-reference" },
            { text: "CLI 参考", link: "/zh-CN/reference/cli" },
            { text: "Profile 格式（英文）", link: "/reference/profile" },
            { text: "环境变量（英文）", link: "/reference/environment" },
            { text: "发布流水线（英文）", link: "/reference/release" }
          ]
        },
        {
          text: "说明",
          items: [
            { text: "为什么选择 envlock（英文）", link: "/explanation/why-envlock" },
            { text: "常见问题", link: "/zh-CN/explanation/faq" },
            { text: "设计边界（英文）", link: "/explanation/design-boundaries" },
            { text: "故障排查（英文）", link: "/explanation/troubleshooting" },
            { text: "支持策略（英文）", link: "/explanation/support-policy" },
            { text: "语言维护（英文）", link: "/explanation/language-maintenance" }
          ]
        }
      ]
    },
    search: {
      provider: "local"
    },
    socialLinks: [{ icon: "github", link: "https://github.com/PerishCode/envlock" }],
    footer: {
      message: "Built with VitePress",
      copyright: "Copyright © 2026 PerishCode"
    }
  }
});
