import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Cargo usage: the registry in .cargo/config.toml (a proxy replaces crates.io), cargo add,
 * Cargo.toml and, for a hosted repository, cargo publish. The repository is a sparse index.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, repositoryName } = context;
  const index = `sparse+${withSlash(context.repositoryUrl)}`;
  const strings = usageStrings();
  const proxy = context.repositoryType === 'proxy';

  const config = proxy
    ? '[source.crates-io]\n' +
      `replace-with = "${repositoryName}"\n\n` +
      `[source.${repositoryName}]\n` +
      `registry = "${index}"`
    : `[registries.${repositoryName}]\nindex = "${index}"`;
  const registryFlag = proxy ? '' : ` --registry ${repositoryName}`;

  const snippets: UsageSnippet[] = [
    { id: 'config', label: 'config.toml', snippetText: config, description: strings.HINTS.CARGO_CONFIG },
  ];
  if (name) {
    snippets.push({
      id: 'add',
      label: 'cargo add',
      snippetText: `cargo add ${name}${version ? `@${version}` : ''}${registryFlag}`,
    });
    snippets.push({
      id: 'manifest',
      label: 'Cargo.toml',
      snippetText: proxy
        ? `[dependencies]\n${name} = "${version ?? '*'}"`
        : `[dependencies]\n${name} = { version = "${version ?? '*'}", registry = "${repositoryName}" }`,
    });
  }
  if (isHosted(context.repositoryType)) {
    snippets.push({
      id: 'publish',
      label: 'cargo publish',
      snippetText: `cargo login --registry ${repositoryName}\ncargo publish --registry ${repositoryName}`,
      description: strings.HINTS.CARGO_PUBLISH,
    });
  }
  return snippets;
};
