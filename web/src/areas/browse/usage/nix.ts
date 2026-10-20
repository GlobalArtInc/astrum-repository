import type { Snippet, UsageComponent, UsageContext } from './snippets';

export const NIX_STORE = '/nix/store';

export function storePath(component: UsageComponent | undefined, store = NIX_STORE): string | null {
  if (!component?.group || !component.name) {
    return null;
  }
  const name = component.version ? `${component.name}-${component.version}` : component.name;
  return `${store}/${component.group}-${name}`;
}

export function copyFrom(url: string, path: string): string {
  return `nix copy --from ${url} ${path}`;
}

function hostOf(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return url.replace(/^[a-z]+:\/\//i, '').replace(/[:/].*$/, '');
  }
}

function attribute(context: UsageContext, key: string): string | undefined {
  const value = context.attributes?.[key];
  return typeof value === 'string' && value !== '' ? value : undefined;
}

export function nix(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const snippets: Snippet[] = [
    {
      id: 'config',
      label: 'nix.conf',
      code: [
        `substituters = ${url}`,
        'trusted-public-keys = <key name>:<public key>',
        'netrc-file = /etc/nix/netrc',
      ].join('\n'),
      hint: 'browse.usage.hint.nixConfig',
    },
    {
      id: 'netrc',
      label: '/etc/nix/netrc',
      code: [`machine ${hostOf(url)}`, 'login <user>', 'password <access token>'].join('\n'),
      hint: 'browse.usage.hint.nixNetrc',
    },
  ];
  const path = attribute(context, 'storePath') ?? storePath(context.component);
  if (path) {
    snippets.push({ id: 'copy-from', label: 'nix copy', code: copyFrom(url, path) });
  }
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'nix copy',
      labelKey: 'browse.usage.label.upload',
      code: [
        'nix key generate-secret --key-name <key name> > secret-key',
        'nix key convert-secret-to-public < secret-key',
        `nix copy --to '${url}?secret-key=secret-key' ./result`,
      ].join('\n'),
      hint: 'browse.usage.hint.nixPublish',
    });
  }
  return snippets;
}
