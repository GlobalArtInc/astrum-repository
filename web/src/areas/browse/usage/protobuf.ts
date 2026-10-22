import type { Snippet, UsageContext } from './snippets';

function registryHost(context: UsageContext): string {
  return context.domain?.routing === 'ROOT' ? context.domain.host : '<registry host>';
}

export function moduleName(context: UsageContext): string {
  const owner = context.component?.group || '<owner>';
  const module = context.component?.name || '<module>';
  return `${registryHost(context)}/${owner}/${module}`;
}

export function protobuf(context: UsageContext): Snippet[] {
  const host = registryHost(context);
  const name = moduleName(context);
  const commit = context.component?.version ?? '';
  const reference = commit === '' ? name : `${name}:${commit}`;
  return [
    {
      id: 'login',
      label: 'Login',
      code: `export BUF_TOKEN=<access token>\nbuf registry login ${host} --username <user name> --token-stdin`,
      hint: 'browse.usage.hint.protobufLogin',
    },
    {
      id: 'module',
      label: 'buf.yaml',
      code: ['version: v2', 'modules:', '  - path: .', `    name: ${name}`].join('\n'),
      hint: 'browse.usage.hint.protobufModule',
    },
    {
      id: 'push',
      label: 'buf push',
      code: 'buf push --create --create-visibility private',
      hint: 'browse.usage.hint.protobufPush',
    },
    {
      id: 'dependency',
      label: 'Dependency',
      code: `version: v2\ndeps:\n  - ${name}\n\nbuf dep update`,
      hint: 'browse.usage.hint.protobufDependency',
    },
    { id: 'export', label: 'buf export', code: `buf export ${reference} --output proto` },
  ];
}
