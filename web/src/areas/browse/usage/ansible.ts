import type { Snippet, UsageComponent, UsageContext } from './snippets';

const ARCHIVE = /^([a-z0-9_]+)-([a-z0-9_]+)-(.+)\.tar\.gz$/;

export function serverName(repository: string): string {
  return repository.replace(/[^A-Za-z0-9_]/g, '_');
}

export function collectionName(component: UsageComponent | undefined): string | null {
  if (!component?.name) {
    return null;
  }
  return component.group ? `${component.group}.${component.name}` : component.name;
}

function archiveName(context: UsageContext): string {
  const last = (context.path ?? '').split('/').pop() ?? '';
  if (ARCHIVE.test(last)) {
    return last;
  }
  const component = context.component;
  if (component?.name && component.group && component.version) {
    return `${component.group}-${component.name}-${component.version}.tar.gz`;
  }
  return '<namespace>-<name>-<version>.tar.gz';
}

export function installCommand(url: string, collection: string, version: string): string {
  return `ansible-galaxy collection install ${collection}${version ? `:${version}` : ''} -s ${url}`;
}

export function requirementsFile(url: string, collection: string, version: string): string {
  return [
    'collections:',
    `  - name: ${collection}`,
    `    version: "${version || '*'}"`,
    `    source: ${url}`,
  ].join('\n');
}

export function ansible(context: UsageContext, url: string): Snippet[] {
  const server = serverName(context.repositoryName);
  const collection = collectionName(context.component) ?? '<namespace>.<name>';
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'config',
      label: 'ansible.cfg',
      code: [
        '[galaxy]',
        `server_list = ${server}`,
        '',
        `[galaxy_server.${server}]`,
        `url = ${url}`,
        'token = <access token>',
      ].join('\n'),
      hint: 'browse.usage.hint.ansibleConfig',
    },
    {
      id: 'install',
      label: 'ansible-galaxy',
      code: installCommand(url, collection, version),
    },
    {
      id: 'requirements',
      label: 'requirements.yml',
      code: requirementsFile(url, collection, version),
      hint: 'browse.usage.hint.ansibleRequirements',
    },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'ansible-galaxy',
      labelKey: 'browse.usage.label.upload',
      code: `ansible-galaxy collection publish ${archiveName(context)} -s ${url} --token <access token>`,
      hint: 'browse.usage.hint.ansiblePublish',
    });
  }
  return snippets;
}
