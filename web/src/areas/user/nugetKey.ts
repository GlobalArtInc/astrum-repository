import { INTERNAL } from '../../api';

export const NUGET_KEY = `${INTERNAL}/ui/user/nuget-api-key`;
export const NUGET_KEY_RESET = `${INTERNAL}/ui/user/nuget-api-key/reset`;
export const KEY_PLACEHOLDER = '{API_KEY}';
export const REPOSITORY_PLACEHOLDER = '{your-nuget-repo}';
export const SOURCE_NAME = 'Astrum';

export interface RevealedKey {
  apiKey: string;
  createdAt: string;
}

export interface KeySnippet {
  id: 'dotnet' | 'nuget' | 'choco';
  label: string;
  code: string;
}

export function sourceUrl(baseUrl: string): string {
  return `${baseUrl.replace(/\/+$/, '')}/repository/${REPOSITORY_PLACEHOLDER}/index.json`;
}

export function dotnetSnippet(baseUrl: string, userId: string, key: string | null): string {
  return [
    `dotnet nuget add source "${SOURCE_NAME}" \\`,
    `  --source ${sourceUrl(baseUrl)} \\`,
    `  --username ${userId} \\`,
    `  --password ${key ?? KEY_PLACEHOLDER}`,
  ].join('\n');
}

export function keySnippets(baseUrl: string, userId: string, key: string | null): KeySnippet[] {
  const secret = key ?? KEY_PLACEHOLDER;
  return [
    { id: 'dotnet', label: 'user.nugetKey.tabs.dotnet', code: dotnetSnippet(baseUrl, userId, key) },
    {
      id: 'nuget',
      label: 'user.nugetKey.tabs.nuget',
      code: [`nuget.exe setapikey ${secret} \\`, `  -Source ${sourceUrl(baseUrl)}`].join('\n'),
    },
    {
      id: 'choco',
      label: 'user.nugetKey.tabs.choco',
      code: [
        `choco source add --name="${SOURCE_NAME}" \\`,
        `  --source="${sourceUrl(baseUrl)}" \\`,
        `  --user="${userId}" \\`,
        `  --password="${secret}"`,
      ].join('\n'),
    },
  ];
}
