import {ROUTE_NAMES} from '../../routerConfig/routeNames/routeNames';

export const RECENT_SEARCHES_KEY = 'astrum-recent-searches';
export const MAX_RECENT_SEARCHES = 5;
export const MAX_REPOSITORIES = 5;
export const MAX_COMPONENTS = 6;

/**
 * The queries the user ran last, newest first. Storage that cannot be read, or holds something else than a list
 * of strings, reads as no history.
 */
export function readRecentSearches() {
  try {
    const stored = JSON.parse(window.localStorage.getItem(RECENT_SEARCHES_KEY) || '[]');
    return Array.isArray(stored)
      ? stored.filter((query) => typeof query === 'string' && query.trim()).slice(0, MAX_RECENT_SEARCHES)
      : [];
  } catch {
    return [];
  }
}

function writeRecentSearches(queries) {
  try {
    window.localStorage.setItem(RECENT_SEARCHES_KEY, JSON.stringify(queries));
  } catch {
    // Storage unavailable or full: the history only lives as long as the page
  }
  return queries;
}

/**
 * Put a query first in the history. The same query typed in another case replaces the older one.
 */
export function addRecentSearch(query) {
  const trimmed = query.trim();
  if (!trimmed) {
    return readRecentSearches();
  }
  const others = readRecentSearches().filter((recent) => recent.toLowerCase() !== trimmed.toLowerCase());
  return writeRecentSearches([trimmed, ...others].slice(0, MAX_RECENT_SEARCHES));
}

export function removeRecentSearch(query) {
  return writeRecentSearches(readRecentSearches().filter((recent) => recent !== query));
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Split a text into the parts that match the query, case-insensitive, and the parts in between.
 */
export function highlightParts(text, query) {
  const value = text == null ? '' : String(text);
  const needle = query.trim();
  if (!value || !needle) {
    return [{text: value, match: false}];
  }
  return value
    .split(new RegExp(`(${escapeRegExp(needle)})`, 'gi'))
    .filter(Boolean)
    .map((part) => ({text: part, match: part.toLowerCase() === needle.toLowerCase()}));
}

/**
 * Up to {@link MAX_REPOSITORIES} repositories whose name contains the query, those that start with it first.
 */
export function matchRepositories(repositories, query) {
  const needle = query.trim().toLowerCase();
  if (!needle) {
    return [];
  }
  return repositories
    .filter((repository) => repository.name?.toLowerCase().includes(needle))
    .sort((a, b) => {
      const aFirst = a.name.toLowerCase().startsWith(needle) ? 0 : 1;
      const bFirst = b.name.toLowerCase().startsWith(needle) ? 0 : 1;
      return aFirst - bFirst || a.name.localeCompare(b.name);
    })
    .slice(0, MAX_REPOSITORIES);
}

function decode(text) {
  try {
    return decodeURIComponent(text);
  } catch {
    return text;
  }
}

/**
 * The repository the current page browses: Astrum One `#browse/<name>/...` or Classic
 * `#browse/browse:<name>[:<path>]`. Null anywhere else.
 */
export function currentRepositoryOf(stateName = '', params = {}) {
  if (stateName === 'preview.browse.browse.repo' || stateName.startsWith('preview.browse.browse.repo.')) {
    return params.repoName || null;
  }
  if (stateName === ROUTE_NAMES.BROWSE.BROWSE.ROOT && params.repo) {
    const [name] = String(params.repo).replace(/^:/, '').split(':');
    return name ? decode(name) : null;
  }
  return null;
}

/**
 * The `keyword` parameter of the Classic search route: the criteria of the search, joined by ` AND ` as the
 * Classic search writes its bookmarks.
 */
export function classicSearchKeyword(query, repository) {
  if (!query) {
    return null;
  }
  const keyword = `=keyword=${encodeURIComponent(query)}`;
  return repository ? `${keyword}${encodeURIComponent(' AND ')}repository_name=${encodeURIComponent(repository)}` : keyword;
}

/**
 * The `repo` parameter of the Classic browse route: the repository, and the path of a node inside it.
 */
export function classicBrowseParam(repository, path) {
  const node = path ? path.replace(/^\/+/, '') : '';
  return `:${encodeURIComponent(repository)}${node ? `:${encodeURIComponent(node)}` : ''}`;
}

/**
 * A component of the search REST API as a row of the panel. The path of its first asset opens it in the browse
 * page, as the search page does.
 */
export function toComponentResult(item) {
  const path = item.assets?.[0]?.path;
  return {
    id: item.id,
    name: item.name,
    group: item.group || null,
    version: item.version || null,
    format: item.format,
    repository: item.repository,
    path: path ? path.replace(/^\/+/, '') : null,
  };
}
