import type { Snippet, UsageComponent, UsageContext } from './snippets';

export function extensionId(component: UsageComponent | undefined): string | null {
  if (!component?.name) {
    return null;
  }
  return component.group ? `${component.group}.${component.name}` : component.name;
}

export function galleryAddresses(url: string): Record<string, string> {
  return {
    serviceUrl: `${url}/vscode/gallery`,
    itemUrl: `${url}/vscode/item`,
    resourceUrlTemplate: `${url}/vscode/unpkg/{publisher}/{name}/{version}/{path}`,
    extensionUrlTemplate: `${url}/vscode/gallery/{publisher}/{name}/latest`,
  };
}

export function productJson(url: string): string {
  return JSON.stringify({ extensionsGallery: galleryAddresses(url) }, null, 2);
}

export function galleryEnvironment(url: string): string {
  const addresses = galleryAddresses(url);
  return [
    `export VSCODE_GALLERY_SERVICE_URL='${addresses.serviceUrl}'`,
    `export VSCODE_GALLERY_ITEM_URL='${addresses.itemUrl}'`,
    `export VSCODE_GALLERY_EXTENSION_URL_TEMPLATE='${addresses.extensionUrlTemplate}'`,
    `export VSCODE_GALLERY_RESOURCE_URL_TEMPLATE='${addresses.resourceUrlTemplate}'`,
  ].join('\n');
}

export function getCommand(url: string, id: string, version: string): string {
  return `npx ovsx get ${id}${version ? ` -v ${version}` : ''} -r ${url}`;
}

function packageName(context: UsageContext): string {
  const last = (context.path ?? '').split('/').pop() ?? '';
  return last.endsWith('.vsix') ? last : '<extension>.vsix';
}

export function openvsx(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const id = extensionId(context.component) ?? '<namespace>.<name>';
  const snippets: Snippet[] = [
    {
      id: 'product',
      label: 'product.json',
      code: productJson(url),
      hint: 'browse.usage.hint.openvsxProduct',
    },
    {
      id: 'environment',
      label: 'VSCODE_GALLERY_*',
      code: galleryEnvironment(url),
      hint: 'browse.usage.hint.openvsxEnvironment',
    },
    {
      id: 'get',
      label: 'ovsx get',
      code: getCommand(url, id, context.component?.version ?? ''),
    },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'ovsx publish',
      labelKey: 'browse.usage.label.upload',
      code: `npx ovsx publish ${packageName(context)} -r ${url} -p <access token>`,
      hint: 'browse.usage.hint.openvsxPublish',
    });
  }
  return snippets;
}
