import { defineConfig } from "vitepress";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const BASE = process.env.RUNSEAL_DOCS_BASE ?? "/";
const THIS_DIR = fileURLToPath(new URL(".", import.meta.url));
const ROOT_FAVICON = readFileSync(resolve(THIS_DIR, "../public/favicon.ico"));

const rootFaviconPlugin = {
  name: "runseal-root-favicon",
  configureServer(server: { middlewares: { use: (path: string, handler: (_req: unknown, res: { setHeader: (name: string, value: string) => void; end: (buffer: Buffer) => void; }) => void) => void; }; }) {
    server.middlewares.use("/favicon.ico", (_req, res) => {
      res.setHeader("Content-Type", "image/x-icon");
      res.end(ROOT_FAVICON);
    });
  },
  configurePreviewServer(server: { middlewares: { use: (path: string, handler: (_req: unknown, res: { setHeader: (name: string, value: string) => void; end: (buffer: Buffer) => void; }) => void) => void; }; }) {
    server.middlewares.use("/favicon.ico", (_req, res) => {
      res.setHeader("Content-Type", "image/x-icon");
      res.end(ROOT_FAVICON);
    });
  }
};

export default defineConfig({
  title: "runseal",
  description: "Seal the run.",
  base: BASE,
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: `${BASE}favicon.svg` }],
    ["link", { rel: "icon", type: "image/png", href: `${BASE}favicon.png` }],
    ["link", { rel: "icon", type: "image/x-icon", href: `${BASE}favicon.ico` }],
    ["link", { rel: "stylesheet", href: `${BASE}home-layout-shell.css` }],
    ["script", { type: "module", src: `${BASE}home-layout-wc.js` }],
    ["meta", { name: "author", content: "PerishCode" }],
    ["meta", { name: "copyright", content: "Copyright © 2026 PerishCode" }],
    ["meta", { name: "agent:owner", content: "PerishCode" }],
    ["meta", { name: "agent:project", content: "runseal" }],
    ["meta", { name: "agent:contract:version", content: "1" }],
    [
      "meta",
      {
        name: "agent:index:v1",
        content: "agent:contract:version,agent:mode,agent:entry:install,agent:entry:use,agent:entry:scoreboard,agent:resolution,agent:locale:default,agent:locale:source,agent:locale:policy"
      }
    ],
    ["meta", { name: "agent:mode", content: "meta-first" }],
    ["meta", { name: "agent:resolution", content: "meta-primary_dom-secondary_dom-wins-on-conflict" }],
    ["meta", { name: "agent:locale:default", content: "en-US" }],
    ["meta", { name: "agent:locale:source", content: "html_lang" }],
    ["meta", { name: "agent:locale:policy", content: "canonical-entries-en-US" }],
    [
      "meta",
      {
        name: "agent:entry:install",
        content: `${BASE}how-to/install`
      }
    ],
    [
      "meta",
      {
        name: "agent:entry:use",
        content: `${BASE}how-to/use-profiles`
      }
    ],
    [
      "meta",
      {
        name: "agent:entry:scoreboard",
        content: `${BASE}explanation/runseal-score/native`
      }
    ]
  ],
  cleanUrls: true,
  lastUpdated: true,
  vite: {
    plugins: [rootFaviconPlugin],
    vue: {
      template: {
        compilerOptions: {
          isCustomElement: (tag: string) => tag.startsWith("wc-")
        }
      }
    }
  },
  locales: {
    root: {
      lang: "en-US",
      label: "English",
      link: "/",
      title: "runseal",
      description: "Seal the run.",
      themeConfig: {
        nav: [
          { text: "Install", link: "/how-to/install" },
          { text: "Use", link: "/how-to/use-profiles" },
          { text: "Posts", link: "/posts/what-is-runseal" },
          { text: ":node/", link: "/node/01-making-npm-i-g-pnpm-sealable" },
          { text: "FAQ", link: "/explanation/faq" },
          { text: "Scoreboard", link: "/explanation/runseal-score/native" },
          { text: "GitHub", link: "https://github.com/PerishCode/runseal" }
        ],
        outline: {
          level: [2, 3],
          label: "On this page"
        },
        editLink: {
          pattern: "https://github.com/PerishCode/runseal/edit/main/docs/:path",
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
      title: "runseal",
      description: "Seal the run.",
      themeConfig: {
        nav: [
          { text: "安装", link: "/zh-CN/how-to/install" },
          { text: "使用", link: "/zh-CN/how-to/use-profiles" },
          { text: "Posts", link: "/zh-CN/posts/what-is-runseal" },
          { text: ":node/", link: "/zh-CN/node/01-making-npm-i-g-pnpm-sealable" },
          { text: "FAQ", link: "/zh-CN/explanation/faq" },
          { text: "Scoreboard", link: "/zh-CN/explanation/runseal-score/native" },
          { text: "GitHub", link: "https://github.com/PerishCode/runseal" }
        ],
        outline: {
          level: [2, 3],
          label: "本页导航"
        },
        editLink: {
          pattern: "https://github.com/PerishCode/runseal/edit/main/docs/:path",
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
    logo: "/favicon.svg",
    sidebar: {
      "/": [
        {
          text: "Docs",
          items: [
            { text: "Install", link: "/how-to/install" },
            { text: "Use Profiles", link: "/how-to/use-profiles" },
            { text: "FAQ", link: "/explanation/faq" }
          ]
        },
        {
          text: "Scoreboard",
          items: [
            { text: "L4 Native", link: "/explanation/runseal-score/native" },
            { text: "L3 Good", link: "/explanation/runseal-score/good" },
            { text: "L2 Normal", link: "/explanation/runseal-score/normal" },
            { text: "L1 Other", link: "/explanation/runseal-score/other" }
          ]
        },
        {
          text: "Posts",
          items: [
            { text: "What is runseal?", link: "/posts/what-is-runseal" },
            { text: "How We Want to Build runseal", link: "/posts/how-we-want-to-build-runseal" },
            { text: "Why We Want to Build runseal", link: "/posts/why-we-want-to-build-runseal" }
          ]
        },
        {
          text: ":node/",
          items: [
            { text: "01 | Making npm i -g pnpm Sealable", link: "/node/01-making-npm-i-g-pnpm-sealable" },
            { text: "02 | What :node Still Needs to Prove", link: "/node/02-what-node-still-needs-to-prove" },
            { text: "03 | Where Corepack Fits", link: "/node/03-where-corepack-fits" },
            { text: "04 | Which Boundaries Are Not Ours", link: "/node/04-which-boundaries-are-not-ours" },
            { text: "05 | Which Node Surface Wins", link: "/node/05-which-node-surface-wins" }
          ]
        }
      ],
      "/zh-CN/": [
        {
          text: "文档",
          items: [
            { text: "安装", link: "/zh-CN/how-to/install" },
            { text: "使用 Profiles", link: "/zh-CN/how-to/use-profiles" },
            { text: "FAQ", link: "/zh-CN/explanation/faq" }
          ]
        },
        {
          text: "Scoreboard",
          items: [
            { text: "L4 Native", link: "/zh-CN/explanation/runseal-score/native" },
            { text: "L3 Good", link: "/zh-CN/explanation/runseal-score/good" },
            { text: "L2 Normal", link: "/zh-CN/explanation/runseal-score/normal" },
            { text: "L1 Other", link: "/zh-CN/explanation/runseal-score/other" }
          ]
        },
        {
          text: "Posts",
          items: [
            { text: "What is runseal?", link: "/zh-CN/posts/what-is-runseal" },
            { text: "How We Want to Build runseal", link: "/zh-CN/posts/how-we-want-to-build-runseal" },
            { text: "Why We Want to Build runseal", link: "/zh-CN/posts/why-we-want-to-build-runseal" }
          ]
        },
        {
          text: ":node/",
          items: [
            { text: "01 | Making npm i -g pnpm Sealable", link: "/zh-CN/node/01-making-npm-i-g-pnpm-sealable" },
            { text: "02 | What :node Still Needs to Prove", link: "/zh-CN/node/02-what-node-still-needs-to-prove" },
            { text: "03 | Where Corepack Fits", link: "/zh-CN/node/03-where-corepack-fits" },
            { text: "04 | Which Boundaries Are Not Ours", link: "/zh-CN/node/04-which-boundaries-are-not-ours" },
            { text: "05 | Which Node Surface Wins", link: "/zh-CN/node/05-which-node-surface-wins" }
          ]
        }
      ]
    },
    search: {
      provider: "local"
    },
    socialLinks: [{ icon: "github", link: "https://github.com/PerishCode/runseal" }]
  }
});
