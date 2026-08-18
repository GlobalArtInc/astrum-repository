

import { generate as cargo } from './cargo';
import { generate as composer } from './composer';
import { generate as docker } from './docker';
import { generate as maven2 } from './maven2';
import { generate as npm } from './npm';
import { generate as nuget } from './nuget';
import { generate as oci } from './oci';
import { generate as pypi } from './pypi';
import { generate as rubygems } from './rubygems';
import { generate as swift } from './swift';
import { generate as terraform } from './terraform';
import type { DependencySnippet, SnippetAssetModel, SnippetComponentModel, SnippetGenerator } from './types';

/**
 * Per-format snippet generators, keyed by the Classic registry key (mirrors the ExtJS
 * `addDependencySnippetGenerator(<key>, ...)` registrations). Maven registers under `maven2`.
 */
const GENERATORS: Readonly<Record<string, SnippetGenerator>> = {
  cargo,
  composer,
  docker,
  maven2,
  npm,
  nuget,
  oci,
  pypi,
  rubygems,
  swift,
  terraform,
};

/**
 * Aliases mapping a raw repository format to the registry key of its generator.
 * The search API reports Maven as `maven`, while the Classic generator is keyed `maven2`.
 *
 * Shared with `formatLabel.ts`, which keys the human-readable label map the same way.
 */
export const FORMAT_ALIASES: Readonly<Record<string, string>> = {
  maven: 'maven2',
};

/**
 * Normalize a raw repository format to the registry key used by the generators and the
 * Classic analytics endpoint (e.g. `maven` -> `maven2`). Lowercases and applies aliases.
 */
export function normalizeFormatKey(format: string): string {
  const key = (format ?? '').toLowerCase();
  return FORMAT_ALIASES[key] ?? key;
}

/**
 * Resolve the dependency snippets for a component of the given repository format.
 *
 * Returns an empty array for a format with no generator, matching Classic's behavior of
 * rendering no snippet section.
 */
export function getDependencySnippets(
  format: string,
  component: SnippetComponentModel,
  asset?: SnippetAssetModel,
): DependencySnippet[] {
  const generator = GENERATORS[normalizeFormatKey(format)];
  return generator ? generator(component, asset) : [];
}
