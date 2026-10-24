import type { Snippet, UsageContext } from './snippets';

const ID_PLACEHOLDER = '<add-on id>';

function trimmed(url: string): string {
  return url.replace(/\/+$/, '');
}

export function addonSegment(id: string): string {
  return encodeURIComponent(id).replace(/%40/g, '@');
}

export function latestUrl(repositoryUrl: string, id: string): string {
  return `${trimmed(repositoryUrl)}/${addonSegment(id)}/latest.xpi`;
}

export function updatesUrl(repositoryUrl: string, id: string): string {
  return `${trimmed(repositoryUrl)}/${addonSegment(id)}/updates.json`;
}

export function versionCheckUrl(repositoryUrl: string): string {
  return `${trimmed(repositoryUrl)}/update/VersionCheck.php?id=%ITEM_ID%&version=%ITEM_VERSION%&appVersion=%APP_VERSION%`;
}

export function downloadCommand(repositoryUrl: string, id: string, version: string): string {
  const file = `${id.replace(/[{}]/g, '').split('@')[0] || 'addon'}-${version}.xpi`;
  return `curl -fLo '${file}' '${trimmed(repositoryUrl)}/${addonSegment(id)}/${encodeURIComponent(version)}.xpi'`;
}

export function policiesJson(repositoryUrl: string, id: string | null, updates: boolean): string {
  const url = trimmed(repositoryUrl);
  const policies: Record<string, unknown> = {
    ExtensionSettings: {
      [id ?? ID_PLACEHOLDER]: {
        installation_mode: 'force_installed',
        install_url: id ? latestUrl(url, id) : `${url}/${ID_PLACEHOLDER}/latest.xpi`,
      },
    },
  };
  if (updates) {
    const check = versionCheckUrl(url);
    policies.Preferences = {
      'extensions.update.url': { Value: check, Status: 'locked' },
      'extensions.update.background.url': { Value: check, Status: 'locked' },
    };
  }
  return JSON.stringify({ policies }, null, 2);
}

export function manifestSettings(repositoryUrl: string, id: string | null): string {
  const settings = {
    browser_specific_settings: {
      gecko: {
        id: id ?? ID_PLACEHOLDER,
        update_url: id
          ? updatesUrl(repositoryUrl, id)
          : `${trimmed(repositoryUrl)}/${ID_PLACEHOLDER}/updates.json`,
      },
    },
  };
  return JSON.stringify(settings, null, 2);
}

export function firefox(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = trimmed(repositoryUrl);
  const kind = context.repositoryType;
  const id = context.component?.name || null;
  const snippets: Snippet[] = [
    {
      id: 'policies',
      label: 'policies.json',
      code: policiesJson(url, id, kind !== 'hosted'),
      hint:
        kind === 'hosted' ? 'browse.usage.hint.firefoxPolicies' : 'browse.usage.hint.firefoxPoliciesProxy',
    },
  ];
  if (kind !== 'proxy') {
    snippets.push({
      id: 'manifest',
      label: 'manifest.json',
      code: manifestSettings(url, id),
      hint: 'browse.usage.hint.firefoxManifest',
    });
  }
  if (context.component?.name) {
    snippets.push({
      id: 'download',
      label: 'curl',
      code: downloadCommand(url, context.component.name, context.component.version),
      hint: 'browse.usage.hint.firefoxDownload',
    });
  }
  if (kind === 'hosted') {
    snippets.push({
      id: 'upload',
      label: 'curl PUT',
      labelKey: 'browse.usage.label.upload',
      code: `curl -i -X PUT -H "Authorization: Bearer <access token>" -T <add-on>.xpi ${url}/<add-on>.xpi`,
      hint: 'browse.usage.hint.firefoxUpload',
    });
  }
  return snippets;
}
