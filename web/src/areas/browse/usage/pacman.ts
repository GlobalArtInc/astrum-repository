import type { Snippet, UsageContext } from './snippets';

const PACKAGE = /^(.+)-([^-]+-[^-]+)-([A-Za-z0-9_]+)\.pkg\.tar(\.[A-Za-z0-9]+)?$/;

export function packageFile(file: string): boolean {
  return PACKAGE.test(file);
}

function segment(value: string): string {
  return encodeURIComponent(value).replace(/%3A/g, ':').replace(/%2B/g, '+').replace(/%40/g, '@');
}

function encoded(segments: readonly string[]): string {
  return segments.map((value) => `${segment(value)}/`).join('');
}

export function pacmanSection(name: string, server: string, trustAll: boolean): string {
  return [`[${name}]`, ...(trustAll ? ['SigLevel = Optional TrustAll'] : []), `Server = ${server}`].join(
    '\n',
  );
}

export function pacman(context: UsageContext, url: string): Snippet[] {
  const segments = (context.path ?? '').split('/').filter((part) => part !== '');
  const stored = segments[segments.length - 1] ?? '';
  const file = stored.replace(/\.sig$/, '');
  const placed = packageFile(file) && segments.length >= 2;
  const above = placed ? segments.slice(0, -2) : [];
  const mirror = above.length === 2 && above[1] === 'os';
  const hosted = context.repositoryType === 'hosted';
  const component = context.component;
  let conf: string;
  let hint: string;
  if (mirror) {
    conf = pacmanSection(above[0] ?? '', `${url}$repo/os/$arch`, hosted);
    hint = 'browse.usage.hint.pacmanMirror';
  } else if (placed || hosted) {
    conf = pacmanSection(context.repositoryName, `${url}${encoded(above)}$arch`, hosted);
    hint = 'browse.usage.hint.pacmanConf';
  } else {
    conf = ['core', 'extra']
      .map((repository) => pacmanSection(repository, `${url}$repo/os/$arch`, false))
      .join('\n\n');
    hint = 'browse.usage.hint.pacmanMirror';
  }
  const snippets: Snippet[] = [
    { id: 'conf', label: '/etc/pacman.conf', code: conf, hint },
    {
      id: 'install',
      label: 'pacman',
      code: component?.name ? `sudo pacman -Sy ${component.name}` : 'sudo pacman -Sy',
    },
  ];
  if (hosted) {
    const name = placed
      ? file
      : component?.name && component.version && component.group
        ? `${component.name}-${component.version}-${component.group}.pkg.tar.zst`
        : '<name>-<version>-<arch>.pkg.tar.zst';
    const target = placed
      ? `${encoded(segments.slice(0, -1))}${segment(file)}`
      : `${component?.group ? segment(component.group) : '<arch>'}/${packageFile(name) ? segment(name) : name}`;
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: [
        `curl -u <user> --upload-file ${name} ${url}${target}`,
        `curl -u <user> --upload-file ${name}.sig ${url}${target}.sig`,
      ].join('\n'),
      hint: 'browse.usage.hint.pacmanUpload',
    });
  }
  return snippets;
}
