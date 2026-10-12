import { usageStrings } from './strings';
import { hostnameOf, isHosted, isPlainHttp, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Swift usage: the registry set with swift package-registry, the dependency in Package.swift, the
 * sign-in and, for a hosted repository, swift package-registry publish. Swift only talks to a plain
 * HTTP registry with --allow-insecure-http and only signs in over HTTPS, so on plain HTTP the
 * credentials go into ~/.netrc instead.
 */
export const generate: UsageGenerator = (context) => {
  const { group, name, version } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const plainHttp = isPlainHttp(url);
  const insecure = plainHttp ? ' --allow-insecure-http' : '';
  const identity = group && name ? `${group}.${name}` : '';

  const snippets: UsageSnippet[] = [
    {
      id: 'registry',
      label: 'swift package-registry',
      snippetText: `swift package-registry set${insecure} ${url}`,
      description: plainHttp ? strings.HINTS.SWIFT_REGISTRY_HTTP : strings.HINTS.SWIFT_REGISTRY,
    },
  ];
  if (identity) {
    snippets.push({
      id: 'manifest',
      label: 'Package.swift',
      snippetText: `.package(id: "${identity}", from: "${version || '1.0.0'}")`,
      description: strings.HINTS.SWIFT_DEPENDENCY,
    });
  }
  snippets.push(plainHttp
    ? {
      id: 'netrc',
      label: '.netrc',
      snippetText: `machine ${hostnameOf(url)}\n  login <user>\n  password <password>`,
      description: strings.HINTS.SWIFT_NETRC,
    }
    : {
      id: 'login',
      label: 'swift package-registry login',
      snippetText: `swift package-registry login ${url} --username <user>`,
      description: strings.HINTS.SWIFT_LOGIN,
    });
  if (isHosted(context.repositoryType)) {
    snippets.push({
      id: 'publish',
      label: 'swift package-registry publish',
      snippetText: `swift package-registry publish ${identity || '<scope>.<name>'} ${version || '<version>'} `
        + `--url ${url}${insecure}`,
      description: strings.HINTS.SWIFT_PUBLISH,
    });
  }
  return snippets;
};
