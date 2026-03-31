import type { Snippet, UsageContext } from './snippets';

export function swift(context: UsageContext, url: string, hostname: string): Snippet[] {
  const component = context.component;
  const plainHttp = /^http:/i.test(url);
  const insecure = plainHttp ? ' --allow-insecure-http' : '';
  const identity = component?.group && component.name ? `${component.group}.${component.name}` : '';
  const version = component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'registry',
      label: 'swift package-registry',
      code: `swift package-registry set${insecure} ${url}`,
      hint: plainHttp ? 'browse.usage.hint.swiftRegistryHttp' : 'browse.usage.hint.swiftRegistry',
    },
  ];
  if (identity) {
    snippets.push({
      id: 'manifest',
      label: 'Package.swift',
      code: `.package(id: "${identity}", from: "${version || '1.0.0'}")`,
      hint: 'browse.usage.hint.swiftDependency',
    });
  }
  snippets.push(
    plainHttp
      ? {
          id: 'netrc',
          label: '.netrc',
          code: `machine ${hostname}\n  login <user>\n  password <password>`,
          hint: 'browse.usage.hint.swiftNetrc',
        }
      : {
          id: 'login',
          label: 'swift package-registry login',
          code: `swift package-registry login ${url} --username <user>`,
          hint: 'browse.usage.hint.swiftLogin',
        },
  );
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'swift package-registry publish',
      code: `swift package-registry publish ${identity || '<scope>.<name>'} ${version || '<version>'} --url ${url}${insecure}`,
      hint: 'browse.usage.hint.swiftPublish',
    });
  }
  return snippets;
}
