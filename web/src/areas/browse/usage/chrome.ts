import type { Snippet, UsageContext } from './snippets';

const PLACEHOLDER = '<extension id>';

function trimmed(url: string): string {
  return url.replace(/\/+$/, '');
}

export function updateUrl(repositoryUrl: string): string {
  return `${trimmed(repositoryUrl)}/updates.xml`;
}

export function extensionSettings(repositoryUrl: string, id: string): string {
  return JSON.stringify(
    {
      ExtensionSettings: {
        [id]: {
          installation_mode: 'force_installed',
          update_url: updateUrl(repositoryUrl),
          override_update_url: true,
        },
      },
    },
    null,
    2,
  );
}

export function forcelist(repositoryUrl: string, id: string): string {
  return JSON.stringify({ ExtensionInstallForcelist: [`${id};${updateUrl(repositoryUrl)}`] }, null, 2);
}

export function registryCommand(repositoryUrl: string, id: string): string {
  return `reg add HKLM\\SOFTWARE\\Policies\\Google\\Chrome\\ExtensionInstallForcelist /v 1 /t REG_SZ /d "${id};${updateUrl(repositoryUrl)}" /f`;
}

export function checkCommand(repositoryUrl: string, id: string): string {
  return `curl '${updateUrl(repositoryUrl)}?response=updatecheck&acceptformat=crx3&x=id%3D${id}%26uc'`;
}

export function downloadCommand(repositoryUrl: string, id: string, version: string): string {
  return `curl -fLO '${trimmed(repositoryUrl)}/${id}/${version}/${id}-${version}.crx'`;
}

export function chrome(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = trimmed(repositoryUrl);
  const id = context.component?.name || PLACEHOLDER;
  const snippets: Snippet[] = [
    {
      id: 'settings',
      label: 'ExtensionSettings',
      code: extensionSettings(url, id),
      hint: 'browse.usage.hint.chromeSettings',
    },
    {
      id: 'forcelist',
      label: 'ExtensionInstallForcelist',
      code: forcelist(url, id),
      hint: 'browse.usage.hint.chromeForcelist',
    },
    {
      id: 'registry',
      label: 'reg add',
      code: registryCommand(url, id),
      hint: 'browse.usage.hint.chromeRegistry',
    },
    {
      id: 'check',
      label: 'curl updatecheck',
      code: checkCommand(url, id),
      hint: 'browse.usage.hint.chromeCheck',
    },
  ];
  if (context.component?.name) {
    snippets.push({
      id: 'download',
      label: 'curl',
      code: downloadCommand(url, context.component.name, context.component.version),
      hint: 'browse.usage.hint.chromeDownload',
    });
  }
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'put',
      label: 'curl PUT',
      code: `curl -i -X PUT -H "Authorization: Bearer <access token>" -T <extension>.crx ${url}/<extension>.crx`,
      hint: 'browse.usage.hint.chromePut',
    });
  }
  return snippets;
}
