/**
 * Types of the repository-aware usage snippets shown in the "Usage" block of a component or asset.
 *
 * Unlike the component-level dependency snippets next to them (ported from the Classic UI), these
 * address the component through the repository it is stored in: the commands carry the
 * repository's own URL, name and registry, so they can be pasted as they are.
 */

/**
 * Everything a usage generator may read about the selected component or asset.
 */
export interface UsageContext {
  /** Raw repository format (e.g. "docker", "maven2", "npm"). */
  readonly format: string;
  /** Name of the repository holding the component. */
  readonly repositoryName: string;
  /** Absolute URL of the repository, e.g. `https://host/repository/npm-hosted/`. */
  readonly repositoryUrl: string;
  /** Repository type: hosted, proxy or group. */
  readonly repositoryType?: string | null;
  /** Namespace of the component (Maven group, npm scope without "@", Composer vendor...). */
  readonly group?: string | null;
  /** Component name. */
  readonly name?: string | null;
  /** Component version. */
  readonly version?: string | null;
  /** Path of the selected asset inside the repository, without a leading slash. */
  readonly path?: string | null;
  /** Docker registry the server computed for the repository (host[:port][/repository]). */
  readonly registryUrl?: string | null;
  /** Format attributes of the selected asset (e.g. `maven2.extension`). */
  readonly attributes?: Readonly<Record<string, unknown>> | null;
  /** APT distribution of the repository, when known. */
  readonly distribution?: string | null;
  /** Directory depth at which a Yum hosted repository generates repodata, when known. */
  readonly repodataDepth?: number | null;
  /** Whether a Yum repository signs its repomd.xml, when known. */
  readonly repodataSigned?: boolean | null;
}

/**
 * One tab of the "Usage" block.
 */
export interface UsageSnippet {
  /** Stable identifier of the tab within its format. */
  readonly id: string;
  /** Tab label, usually the name of the client or file (not translated). */
  readonly label: string;
  /** The copyable text. */
  readonly snippetText: string;
  /** One-line hint shown above the snippet. */
  readonly description?: string;
}

/**
 * A usage generator of one format.
 */
export type UsageGenerator = (context: UsageContext) => UsageSnippet[];
