import type { Snippet, UsageContext } from './snippets';

function trimmed(url: string): string {
  return url.replace(/\/+$/, '');
}

export function updateFileUrl(repositoryUrl: string): string {
  return `${trimmed(repositoryUrl)}/updatePlugins.xml`;
}

export function downloadCommand(repositoryUrl: string, id: string, version: string): string {
  const query = `pluginId=${encodeURIComponent(id)}&version=${encodeURIComponent(version)}`;
  return `curl -fLOJ '${trimmed(repositoryUrl)}/plugin/download?${query}'`;
}

export function jetbrains(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = trimmed(repositoryUrl);
  const kind = context.repositoryType;
  const snippets: Snippet[] = [];
  if (kind !== 'proxy') {
    snippets.push(
      {
        id: 'repository',
        label: 'Manage Plugin Repositories',
        code: updateFileUrl(url),
        hint: 'browse.usage.hint.jetbrainsRepository',
      },
      {
        id: 'properties',
        label: 'idea.plugin.hosts',
        code: `idea.plugin.hosts=${updateFileUrl(url)}`,
        hint: 'browse.usage.hint.jetbrainsHosts',
      },
    );
  }
  if (kind !== 'hosted') {
    snippets.push(
      {
        id: 'host',
        label: 'idea.plugins.host',
        code: `idea.plugins.host=${url}`,
        hint: 'browse.usage.hint.jetbrainsHost',
      },
      {
        id: 'vmoptions',
        label: 'VM options',
        code: `-Didea.plugins.host=${url}`,
        hint: 'browse.usage.hint.jetbrainsVmOptions',
      },
    );
  }
  if (context.component?.name) {
    snippets.push({
      id: 'download',
      label: 'curl',
      code: downloadCommand(url, context.component.name, context.component.version),
      hint: 'browse.usage.hint.jetbrainsDownload',
    });
  }
  if (kind === 'hosted') {
    snippets.push(
      {
        id: 'upload',
        label: 'Upload API',
        labelKey: 'browse.usage.label.upload',
        code: `curl -i -H "Authorization: Bearer <access token>" -F xmlId=<plugin id> -F file=@<plugin>.zip ${url}/api/updates/upload`,
        hint: 'browse.usage.hint.jetbrainsUpload',
      },
      {
        id: 'put',
        label: 'curl PUT',
        code: `curl -i -X PUT -H "Authorization: Bearer <access token>" -T <plugin>.zip ${url}/<plugin>.zip`,
        hint: 'browse.usage.hint.jetbrainsPut',
      },
    );
  }
  return snippets;
}
