
package org.globalart.astrum.repository.search;

import javax.annotation.Nullable;

/**
 * A request for one page of a component's distinct versions.
 *
 * @param format the component format, required
 * @param namespace the component namespace, may be null for formats without one
 * @param name the component name, required
 * @param versionFilter optional version substring filter
 * @param page zero-based page index
 * @param size page size
 * @param sort sort key: version, lastUpdated, or repositories
 * @param direction sort direction
 */
public record ComponentVersionQuery(
    String format,
    @Nullable String namespace,
    String name,
    @Nullable String versionFilter,
    int page,
    int size,
    String sort,
    SortDirection direction)
{
}
