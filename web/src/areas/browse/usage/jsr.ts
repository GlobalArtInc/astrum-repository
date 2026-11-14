import type { Snippet, UsageContext } from './snippets';

export function packageSpecifier(context: UsageContext): string {
  const scope = (context.component?.group || '@<scope>').replace(/^@?/, '@');
  const name = context.component?.name || '<package>';
  const version = context.component?.version;
  return version ? `jsr:${scope}/${name}@${version}` : `jsr:${scope}/${name}`;
}

export function jsr(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = `${repositoryUrl.replace(/\/+$/, '')}/`;
  const specifier = packageSpecifier(context);
  const alias = specifier.replace(/^jsr:/, '').replace(/@[^@/]*$/, '') || specifier;
  const hosted = context.repositoryType === 'hosted';
  return [
    {
      id: 'environment',
      label: 'JSR_URL',
      code: `export JSR_URL=${url}`,
      hint: 'browse.usage.hint.jsrEnvironment',
    },
    {
      id: 'imports',
      label: 'deno.json',
      code: JSON.stringify({ imports: { [alias]: specifier } }, null, 2),
      hint: 'browse.usage.hint.jsrImports',
    },
    ...(hosted
      ? [
          {
            id: 'publish',
            label: 'deno publish',
            code: 'deno publish --token <access token>',
            hint: 'browse.usage.hint.jsrPublish',
          },
        ]
      : []),
  ];
}
