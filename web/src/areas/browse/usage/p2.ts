import type { Snippet, UsageComponent, UsageContext } from './snippets';

function xml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

export function installableUnit(component: UsageComponent | undefined): string | null {
  if (!component?.name) {
    return null;
  }
  return component.group === 'features' ? `${component.name}.feature.group` : component.name;
}

export function directorCommand(url: string, unit: string): string {
  return [
    'eclipse -nosplash -application org.eclipse.equinox.p2.director \\',
    `  -repository ${url} \\`,
    `  -installIU ${unit}`,
  ].join('\n');
}

export function p2(context: UsageContext, url: string): Snippet[] {
  const unit = installableUnit(context.component);
  const version = context.component?.version ?? '';
  const id = xml(context.repositoryName);
  return [
    {
      id: 'eclipse',
      label: 'Eclipse',
      code: `Help > Install New Software... > Add...\nName: ${context.repositoryName}\nLocation: ${url}`,
      hint: 'browse.usage.hint.p2Eclipse',
    },
    {
      id: 'tycho',
      label: 'Tycho',
      code: [
        '<repositories>',
        '  <repository>',
        `    <id>${id}</id>`,
        '    <layout>p2</layout>',
        `    <url>${xml(url)}</url>`,
        '  </repository>',
        '</repositories>',
      ].join('\n'),
      hint: 'browse.usage.hint.p2Tycho',
    },
    {
      id: 'director',
      label: 'p2 director',
      code: directorCommand(url, unit ? `${unit}${version ? `/${version}` : ''}` : '<installable unit>'),
      hint: 'browse.usage.hint.p2Director',
    },
  ];
}
