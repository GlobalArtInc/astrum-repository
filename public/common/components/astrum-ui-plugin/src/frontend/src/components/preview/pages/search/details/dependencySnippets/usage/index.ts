import { getDependencySnippets, normalizeFormatKey } from '../registry';
import { generate as alpine } from './alpine';
import { generate as apt } from './apt';
import { generate as cargo } from './cargo';
import { generate as cocoapods } from './cocoapods';
import { generate as composer } from './composer';
import { generate as conan } from './conan';
import { generate as conda } from './conda';
import { generate as docker } from './docker';
import { generate as gitlfs } from './gitlfs';
import { generate as go } from './go';
import { generate as helm } from './helm';
import { generate as maven2 } from './maven2';
import { generate as npm } from './npm';
import { generate as nuget } from './nuget';
import { generate as p2 } from './p2';
import { generate as pypi } from './pypi';
import { generate as r } from './r';
import { generate as raw } from './raw';
import type { UsageContext, UsageGenerator, UsageSnippet } from './types';

export type { UsageContext, UsageGenerator, UsageSnippet } from './types';

/**
 * Repository-aware usage generators, keyed by the format id the server uses.
 */
export const USAGE_GENERATORS: Readonly<Record<string, UsageGenerator>> = {
  alpine,
  apt,
  cargo,
  cocoapods,
  composer,
  conan,
  conda,
  docker,
  gitlfs,
  go,
  helm,
  maven2,
  npm,
  nuget,
  p2,
  pypi,
  r,
  raw,
};

/**
 * The usage snippets of a component or asset.
 *
 * Formats with a repository-aware generator get commands that carry the repository's URL; any other
 * format falls back to its component-level dependency snippets, and a format with neither gets none.
 */
export function getUsageSnippets(context: UsageContext): UsageSnippet[] {
  const key = normalizeFormatKey(context.format);
  const generator = USAGE_GENERATORS[key];
  if (generator) {
    return generator(context).filter((snippet) => snippet.snippetText.trim() !== '');
  }
  if (!context.name || !context.version) {
    return [];
  }
  return getDependencySnippets(context.format, {
    format: context.format,
    group: context.group ?? '',
    name: context.name,
    version: context.version,
    repositoryName: context.repositoryName,
  }).map((snippet) => ({
    id: snippet.displayName,
    label: snippet.displayName,
    snippetText: snippet.snippetText,
    description: snippet.description,
  }));
}
