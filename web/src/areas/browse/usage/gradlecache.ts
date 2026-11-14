import type { Snippet } from './snippets';

export function gradlecache(repositoryUrl: string): Snippet[] {
  const url = `${repositoryUrl.replace(/\/+$/, '')}/`;
  return [
    {
      id: 'settings',
      label: 'Settings (settings.gradle.kts)',
      code: [
        'buildCache {',
        '    remote<HttpBuildCache> {',
        `        url = uri("${url}")`,
        '        isPush = true',
        '        credentials {',
        '            username = "<user>"',
        '            password = "<access token>"',
        '        }',
        '    }',
        '}',
      ].join('\n'),
      hint: 'browse.usage.hint.gradleSettings',
    },
    { id: 'properties', label: 'gradle.properties', code: 'org.gradle.caching=true' },
    {
      id: 'ci',
      label: 'CI',
      code: './gradlew build --build-cache',
      hint: 'browse.usage.hint.gradleCi',
    },
  ];
}
