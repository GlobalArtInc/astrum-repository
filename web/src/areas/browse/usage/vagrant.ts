import type { Snippet, UsageContext } from './snippets';

const BOX_FILE = /^\/?([^/]+)\/([^/]+)\/([^/]+)\/([^/]+)\/([^/]+)\.box$/;

export function boxName(context: UsageContext): string {
  const component = context.component;
  if (component?.group && component.name) {
    return `${component.group}/${component.name}`;
  }
  const match = BOX_FILE.exec(context.path ?? '');
  return match ? `${match[1]}/${match[2]}` : '<org>/<name>';
}

function versionFlag(version: string): string {
  return version ? ` --box-version ${version}` : '';
}

export function boxAdd(url: string, box: string, version: string): string {
  return `vagrant box add ${url}/${box}${versionFlag(version)}`;
}

export function vagrantfile(url: string, box: string, version: string): string {
  return [
    'Vagrant.configure("2") do |config|',
    `  config.vm.box = "${box}"`,
    `  config.vm.box_url = "${url}/${box}"`,
    ...(version ? [`  config.vm.box_version = "${version}"`] : []),
    'end',
  ].join('\n');
}

function uploadPath(context: UsageContext, box: string, version: string): string {
  const path = context.path ?? '';
  if (BOX_FILE.test(path)) {
    return path.replace(/^\/+/, '');
  }
  return `${box}/${version || '<version>'}/<provider>/<architecture>.box`;
}

export function vagrant(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const box = boxName(context);
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'server',
      label: 'VAGRANT_SERVER_URL',
      code: [
        `export VAGRANT_SERVER_URL=${url}`,
        'export VAGRANT_CLOUD_TOKEN=<access token>',
        `vagrant box add ${box}${versionFlag(version)}`,
      ].join('\n'),
      hint: 'browse.usage.hint.vagrantServer',
    },
    {
      id: 'vagrantfile',
      label: 'Vagrantfile',
      code: vagrantfile(url, box, version),
      hint: 'browse.usage.hint.vagrantBoxUrl',
    },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file package.box ${url}/${uploadPath(context, box, version)}`,
      hint: 'browse.usage.hint.vagrantUpload',
    });
  }
  return snippets;
}
