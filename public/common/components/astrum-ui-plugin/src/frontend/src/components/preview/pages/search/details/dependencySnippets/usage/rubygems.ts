import { usageStrings } from './strings';
import { isHosted, withoutSlash, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * RubyGems usage: gem install from the repository, the Gemfile source, Bundler's mirror of
 * rubygems.org for a proxy and, for a hosted repository, gem push. The repository serves the
 * compact index, so both gem and bundle resolve against it directly.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const snippets: UsageSnippet[] = [];

  if (name) {
    snippets.push({
      id: 'install',
      label: 'gem install',
      snippetText: `gem install ${name}${version ? ` -v ${version}` : ''} --clear-sources --source ${url}`,
    });
  }
  snippets.push({
    id: 'gemfile',
    label: 'Gemfile',
    snippetText: `source "${url}"${name ? `\n\ngem "${name}"${version ? `, "${version}"` : ''}` : ''}`,
    description: strings.HINTS.GEMFILE,
  });
  if (context.repositoryType === 'proxy') {
    snippets.push({
      id: 'mirror',
      label: 'bundle config',
      snippetText: `bundle config set --global mirror.https://rubygems.org ${url}`,
      description: strings.HINTS.BUNDLE_MIRROR,
    });
  } else {
    snippets.push({
      id: 'sources',
      label: 'gem sources',
      snippetText: `gem sources --add ${url}`,
      description: strings.HINTS.GEM_SOURCES,
    });
  }
  if (isHosted(context.repositoryType)) {
    const host = withoutSlash(context.repositoryUrl);
    const file = name && version ? `${name}-${version}.gem` : '<name>-<version>.gem';
    snippets.push({
      id: 'push',
      label: 'gem push',
      snippetText: `export RUBYGEMS_HOST=${host}\ngem signin\ngem push ${file}`,
      description: strings.HINTS.GEM_PUSH,
    });
  }
  return snippets;
};
