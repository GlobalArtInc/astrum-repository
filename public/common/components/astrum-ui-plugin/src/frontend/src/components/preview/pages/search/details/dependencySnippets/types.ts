

/**
 * Dependency snippet types for the Preview UI Overview tab.
 *
 * These types mirror the Classic UI's dependency-snippet contract
 * (AS/model/DependencySnippet.js and the per-format snippet generators) so the two
 * UIs can produce identical snippet text. See details/dependencySnippets/registry.ts.
 */

/**
 * A single copyable dependency snippet, matching the Classic generator return shape.
 */
export interface DependencySnippet {
  /** Short label for the tool/snippet (e.g. "Apache Maven", "npm", "package.json"). */
  readonly displayName: string;
  /** Optional human-readable instruction shown above the snippet text. */
  readonly description?: string;
  /** The copyable snippet body. */
  readonly snippetText: string;
}

/**
 * Component-level coordinates a generator reads from.
 *
 * Mirrors the fields the Classic `componentModel` exposes via `.get(...)`:
 * `format`, `group`, `name`, `version`, and (for NuGet) `repositoryName`.
 */
export interface SnippetComponentModel {
  /** Raw repository format (e.g. "maven2", "npm", "pypi"). */
  readonly format: string;
  /** Namespace/group. For scoped npm this is the scope without the leading "@". */
  readonly group: string;
  /** Component name. */
  readonly name: string;
  /** Component version. May be empty for versionless formats. */
  readonly version: string;
  /** Repository name; used by the NuGet Chocolatey snippet. */
  readonly repositoryName?: string;
}

/**
 * Asset-level attributes a generator may read from.
 *
 * The Overview tab is component-level, so generators are invoked with `undefined`
 * here (matching Classic passing `assetModel` as undefined). The shape is kept so the
 * asset-aware generators (Maven, Docker) can be reused for per-asset snippets later.
 */
export interface SnippetAssetModel {
  readonly attributes?: {
    readonly maven2?: {
      readonly classifier?: string;
      readonly extension?: string;
    };
  };
  /** Docker top-level registry URL (ASTRUM-51972). */
  readonly registryUrl?: string;
}

/**
 * A per-format snippet generator, mirroring a Classic `snippetGenerator(componentModel, assetModel)`.
 */
export type SnippetGenerator = (
  component: SnippetComponentModel,
  asset?: SnippetAssetModel,
) => DependencySnippet[];
