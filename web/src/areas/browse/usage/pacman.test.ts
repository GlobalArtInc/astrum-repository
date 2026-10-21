import { describe, expect, it } from 'vitest';
import { packageFile, pacmanSection } from './pacman';
import { supportsUsage, usageSnippets } from './snippets';

const base = {
  format: 'pacman',
  repositoryName: 'pkgs',
  repositoryType: 'hosted' as const,
  baseUrl: 'https://astrum.example.com',
};
const url = 'https://astrum.example.com/repository/pkgs/';

function code(snippets: ReturnType<typeof usageSnippets>, id: string): string | undefined {
  return snippets.find((snippet) => snippet.id === id)?.code;
}

describe('pacman usage', () => {
  it('configures a hosted repository by architecture and uploads packages with their signatures', () => {
    expect(supportsUsage('pacman')).toBe(true);
    const snippets = usageSnippets(base);
    expect(snippets.map((snippet) => snippet.id)).toEqual(['conf', 'install', 'upload']);
    expect(code(snippets, 'conf')).toBe(`[pkgs]\nSigLevel = Optional TrustAll\nServer = ${url}$arch`);
    expect(snippets[0]?.hint).toBe('browse.usage.hint.pacmanConf');
    expect(code(snippets, 'install')).toBe('sudo pacman -Sy');
    expect(code(snippets, 'upload')).toBe(
      [
        `curl -u <user> --upload-file <name>-<version>-<arch>.pkg.tar.zst ${url}<arch>/<name>-<version>-<arch>.pkg.tar.zst`,
        `curl -u <user> --upload-file <name>-<version>-<arch>.pkg.tar.zst.sig ${url}<arch>/<name>-<version>-<arch>.pkg.tar.zst.sig`,
      ].join('\n'),
    );
    expect(snippets[2]?.hint).toBe('browse.usage.hint.pacmanUpload');
  });

  it('follows the folders of a stored package and names the component', () => {
    const snippets = usageSnippets({
      ...base,
      path: '/custom/os/x86_64/hello-1:1.0-2-x86_64.pkg.tar.xz.sig',
      component: { group: 'x86_64', name: 'hello', version: '1:1.0-2' },
    });
    expect(code(snippets, 'conf')).toBe(
      `[custom]\nSigLevel = Optional TrustAll\nServer = ${url}$repo/os/$arch`,
    );
    expect(snippets[0]?.hint).toBe('browse.usage.hint.pacmanMirror');
    expect(code(snippets, 'install')).toBe('sudo pacman -Sy hello');
    expect(code(snippets, 'upload')?.split('\n')[0]).toBe(
      `curl -u <user> --upload-file hello-1:1.0-2-x86_64.pkg.tar.xz ${url}custom/os/x86_64/hello-1:1.0-2-x86_64.pkg.tar.xz`,
    );
    const flat = usageSnippets({ ...base, path: '/stable/x86_64/tool-2.0-1-x86_64.pkg.tar.zst' });
    expect(code(flat, 'conf')).toBe(`[pkgs]\nSigLevel = Optional TrustAll\nServer = ${url}stable/$arch`);
    const named = usageSnippets({ ...base, component: { group: 'any', name: 'docs', version: '1.0-1' } });
    expect(code(named, 'upload')?.split('\n')[1]).toBe(
      `curl -u <user> --upload-file docs-1.0-1-any.pkg.tar.zst.sig ${url}any/docs-1.0-1-any.pkg.tar.zst.sig`,
    );
  });

  it('shows the mirror layout for proxies and groups without an upload', () => {
    const proxy = usageSnippets({ ...base, repositoryName: 'arch', repositoryType: 'proxy' });
    expect(proxy.map((snippet) => snippet.id)).toEqual(['conf', 'install']);
    const mirror = 'https://astrum.example.com/repository/arch/$repo/os/$arch';
    expect(code(proxy, 'conf')).toBe(`[core]\nServer = ${mirror}\n\n[extra]\nServer = ${mirror}`);
    const group = usageSnippets({
      ...base,
      repositoryName: 'all',
      repositoryType: 'group',
      path: '/x86_64/hello-1.0-1-x86_64.pkg.tar.zst',
    });
    expect(code(group, 'conf')).toBe('[all]\nServer = https://astrum.example.com/repository/all/$arch');
  });

  it('knows package file names', () => {
    expect(packageFile('hello-1.0-1-x86_64.pkg.tar.zst')).toBe(true);
    expect(packageFile('hello-1:1.0-1-any.pkg.tar')).toBe(true);
    expect(packageFile('core.db.tar.gz')).toBe(false);
    expect(pacmanSection('core', 'https://x/$arch', false)).toBe('[core]\nServer = https://x/$arch');
  });
});
