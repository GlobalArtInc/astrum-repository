import { posix } from 'node:path';
import { defineConfig, type DefaultTheme } from 'vitepress';
import { DISMISSED_CLASS, STORAGE_KEY } from './theme/banner';

const REPOSITORY = 'https://github.com/GlobalArtInc/astrum-repository';

const FORMATS: [string, string][] = [
  ['alpine', 'Alpine'],
  ['ansible', 'Ansible Galaxy'],
  ['apt', 'APT'],
  ['bazel', 'Bazel'],
  ['cargo', 'Cargo'],
  ['chef', 'Chef Supermarket'],
  ['chrome', 'Chromium extensions'],
  ['cocoapods', 'CocoaPods'],
  ['composer', 'Composer'],
  ['conan', 'Conan'],
  ['conda', 'Conda'],
  ['docker', 'Docker and OCI'],
  ['firefox', 'Firefox add-ons'],
  ['gitlfs', 'Git LFS'],
  ['go', 'Go'],
  ['helm', 'Helm'],
  ['hex', 'Hex'],
  ['huggingface', 'Hugging Face'],
  ['jetbrains', 'JetBrains plugins'],
  ['maven', 'Maven and Gradle'],
  ['nix', 'Nix'],
  ['npm', 'npm, pnpm, Yarn'],
  ['nuget', 'NuGet'],
  ['nx', 'Nx remote cache'],
  ['openvsx', 'Open VSX'],
  ['p2', 'P2'],
  ['pacman', 'pacman'],
  ['protobuf', 'Protobuf'],
  ['pub', 'pub'],
  ['puppet', 'Puppet Forge'],
  ['pypi', 'PyPI'],
  ['r', 'R'],
  ['raw', 'Raw files'],
  ['rubygems', 'RubyGems'],
  ['swift', 'Swift'],
  ['terraform', 'Terraform'],
  ['turbo', 'Turborepo remote cache'],
  ['vagrant', 'Vagrant'],
  ['yum', 'Yum'],
];

function outsideDocs(href: string, page: string): string | undefined {
  if (!href.startsWith('../')) return undefined;
  const target = posix.normalize(posix.join('docs', posix.dirname(page), href));
  return target.startsWith('docs/') ? undefined : `${REPOSITORY}/blob/main/${target}`;
}

const sidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Introduction',
    items: [{ text: 'Quick start', link: '/getting-started' }],
  },
  {
    text: 'Install',
    items: [
      { text: 'System requirements', link: '/install/requirements' },
      { text: 'Container image', link: '/install/container' },
      { text: 'Without a container', link: '/install/from-source' },
    ],
  },
  {
    text: 'Configure',
    items: [
      { text: 'Runtime', link: '/configure/runtime' },
      { text: 'PostgreSQL', link: '/configure/postgresql' },
      { text: 'Blob stores', link: '/configure/blob-stores' },
      { text: 'Reverse proxy and TLS', link: '/configure/reverse-proxy' },
      { text: 'Users, realms and sign-in', link: '/configure/authentication' },
      { text: 'Outbound HTTP and proxy', link: '/configure/outbound-http' },
      { text: 'SSL certificates', link: '/configure/ssl-certificates' },
      { text: 'Capabilities', link: '/configure/capabilities' },
    ],
  },
  {
    text: 'Repositories',
    items: [
      { text: 'Repositories', link: '/repositories/overview' },
      { text: 'Cleanup policies', link: '/repositories/cleanup-policies' },
      { text: 'Routing rules', link: '/repositories/routing-rules' },
    ],
  },
  {
    text: 'Formats',
    collapsed: true,
    items: [
      { text: 'All formats', link: '/formats/' },
      ...FORMATS.map(([id, text]) => ({ text, link: `/formats/${id}` })),
    ],
  },
  {
    text: 'Use',
    items: [
      { text: 'Web interface', link: '/use/web-interface' },
      { text: 'Connect build tools', link: '/use/clients' },
      { text: 'REST API', link: '/use/rest-api' },
      { text: 'Agents over MCP', link: '/use/mcp' },
      { text: 'Interface language', link: '/use/language' },
    ],
  },
  {
    text: 'Operate',
    items: [
      { text: 'Tasks', link: '/operate/tasks' },
      { text: 'Nodes', link: '/operate/nodes' },
      { text: 'Monitoring and logs', link: '/operate/monitoring' },
      { text: 'Backup and restore', link: '/operate/backup-restore' },
      { text: 'Upgrade', link: '/operate/upgrade' },
    ],
  },
  {
    text: 'Migrate',
    items: [{ text: 'From another server', link: '/migrate/from-another-server' }],
  },
];

export default defineConfig({
  lang: 'en-US',
  title: 'Astrum Repository',
  description:
    'A self-hosted artifact repository manager: hosted, proxy and group repositories for 39 formats in one server.',
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: ['README.md'],
  rewrites: {
    'formats/README.md': 'formats/index.md',
  },
  sitemap: { hostname: 'https://astrum.globalart.dev' },
  ignoreDeadLinks: 'localhostLinks',
  markdown: {
    config(md) {
      md.core.ruler.after('inline', 'links-outside-docs', (state) => {
        const page: string = state.env.relativePath ?? '';
        for (const block of state.tokens) {
          for (const token of block.children ?? []) {
            if (token.type !== 'link_open') continue;
            const href = token.attrGet('href');
            const external = href ? outsideDocs(href, page) : undefined;
            if (external) token.attrSet('href', external);
          }
        }
      });
    },
  },
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['link', { rel: 'icon', href: '/favicon.ico', sizes: '32x32' }],
    ['link', { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' }],
    ['meta', { name: 'theme-color', content: '#5b47e0' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'Astrum Repository' }],
    ['script', {}, `try{if(localStorage.getItem('${STORAGE_KEY}')==='1')document.documentElement.classList.add('${DISMISSED_CLASS}')}catch{}`],
  ],
  themeConfig: {
    logo: { src: '/favicon.svg', alt: '' },
    nav: [
      { text: 'Guide', link: '/getting-started', activeMatch: '^/(getting-started|install|configure|repositories|use|operate|migrate)/?' },
      { text: 'Formats', link: '/formats/', activeMatch: '^/formats/' },
      { text: 'REST API', link: '/use/rest-api' },
      { text: 'Releases', link: `${REPOSITORY}/releases` },
    ],
    sidebar,
    socialLinks: [{ icon: 'github', link: REPOSITORY }],
    search: { provider: 'local' },
    outline: { level: [2, 3] },
    editLink: {
      pattern: ({ filePath }) => `https://github.com/GlobalArtInc/astrum-repository/edit/main/docs/${filePath}`,
      text: 'Edit this page on GitHub',
    },
    footer: {
      message: 'Released under the MIT License.',
      copyright: '© 2026 GlobalArt, Inc.',
    },
  },
});
