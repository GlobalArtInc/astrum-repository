import type { Snippet, UsageContext } from './snippets';

interface Address {
  kind: 'module' | 'provider';
  parts: string[];
}

function addressOf(context: UsageContext): Address | null {
  const path = (context.path ?? '').replace(/^\/+/, '');
  const fromPath = /^v1\/(modules|providers)\/(.+)$/.exec(path);
  if (fromPath?.[1] && fromPath[2]) {
    const segments = fromPath[2].split('/');
    return fromPath[1] === 'modules'
      ? { kind: 'module', parts: segments.slice(0, 3) }
      : { kind: 'provider', parts: segments.slice(0, 2) };
  }
  const component = context.component;
  if (!component?.group) {
    return null;
  }
  const parts = component.name.split('/').filter(Boolean);
  if (parts.length === 2) {
    return { kind: 'module', parts: [component.group, ...parts] };
  }
  if (parts.length === 1) {
    return { kind: 'provider', parts: [component.group, ...parts] };
  }
  return null;
}

export function terraform(context: UsageContext, url: string, host: string): Snippet[] {
  const address = addressOf(context);
  if (!address || address.parts.length < (address.kind === 'module' ? 3 : 2)) {
    return [];
  }
  const version = context.component?.version ?? '';
  const credentials: Snippet = {
    id: 'credentials',
    label: '.terraformrc',
    code: `# curl -u <user> ${url}token\ncredentials "${host}" {\n  token = "<token>"\n}`,
    hint: 'browse.usage.hint.terraformCredentials',
  };
  const hosted = context.repositoryType === 'hosted';
  if (address.kind === 'module') {
    const [namespace, name, system] = address.parts;
    const snippets: Snippet[] = [
      {
        id: 'module',
        label: 'main.tf',
        code: `module "${name}" {\n  source  = "${host}/${namespace}/${name}/${system}"${version ? `\n  version = "${version}"` : ''}\n}`,
        hint: 'browse.usage.hint.terraformRegistry',
        hintVars: { host },
      },
      credentials,
    ];
    if (hosted && version) {
      const file = `${name}-${system}-${version}.tar.gz`;
      snippets.push({
        id: 'upload',
        label: 'curl',
        labelKey: 'browse.usage.label.upload',
        code: `curl -u <user> --upload-file ${file} ${url}v1/modules/${namespace}/${name}/${system}/${version}/${file}`,
        hint: 'browse.usage.hint.terraformUpload',
      });
    }
    return snippets;
  }
  const [namespace, type] = address.parts;
  const proxy = context.repositoryType === 'proxy';
  const source = proxy ? `${namespace}/${type}` : `${host}/${namespace}/${type}`;
  const snippets: Snippet[] = [
    {
      id: 'provider',
      label: 'main.tf',
      code: [
        'terraform {',
        '  required_providers {',
        `    ${type} = {`,
        `      source  = "${source}"${version ? `\n      version = "${version}"` : ''}`,
        '    }',
        '  }',
        '}',
      ].join('\n'),
      hint: proxy ? 'browse.usage.hint.terraformProviderProxy' : 'browse.usage.hint.terraformRegistry',
      hintVars: proxy ? undefined : { host },
    },
    {
      id: 'mirror',
      label: 'network_mirror',
      code: `provider_installation {\n  network_mirror {\n    url = "${url}"\n  }\n}`,
      hint: 'browse.usage.hint.terraformMirror',
    },
    credentials,
  ];
  if (hosted && version) {
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: [
        `for file in terraform-provider-${type}_${version}_*; do`,
        `  curl -u <user> --upload-file "$file" ${url}v1/providers/${namespace}/${type}/${version}/"$file"`,
        'done',
      ].join('\n'),
      hint: 'browse.usage.hint.terraformUpload',
    });
  }
  return snippets;
}
