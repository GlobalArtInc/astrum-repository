import type { Snippet, UsageContext } from './snippets';

export function pubConstraint(version: string): string {
  return version ? `^${version}` : 'any';
}

export function pubDependency(name: string, url: string, version: string): string {
  return ['dependencies:', `  ${name}:`, `    hosted: ${url}`, `    version: ${pubConstraint(version)}`].join(
    '\n',
  );
}

export function pubAdd(name: string, url: string, version: string): string {
  return `dart pub add '${name}:{hosted: ${url}, version: ${pubConstraint(version)}}'`;
}

export function pub(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'token',
      label: 'dart pub token add',
      code: `dart pub token add ${url}`,
      hint: 'browse.usage.hint.pubToken',
    },
    {
      id: 'dependency',
      label: 'pubspec.yaml',
      code: pubDependency(name || '<package>', url, version),
      hint: 'browse.usage.hint.pubDependency',
    },
  ];
  if (name) {
    snippets.push({ id: 'add', label: 'dart pub add', code: pubAdd(name, url, version) });
  }
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'dart pub publish',
      code: `publish_to: ${url}`,
      hint: 'browse.usage.hint.pubPublish',
    });
  } else {
    snippets.push({
      id: 'mirror',
      label: 'PUB_HOSTED_URL',
      code: `export PUB_HOSTED_URL=${url}\ndart pub get`,
      hint: 'browse.usage.hint.pubHostedUrl',
    });
  }
  return snippets;
}
