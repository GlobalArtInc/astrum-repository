
package org.globalart.astrum.repository.search;

import java.util.List;

/**
 * One page of a component's distinct versions.
 *
 * @param items the versions on this page
 * @param total the total number of distinct versions matching the query
 * @param page zero-based page index
 * @param size page size
 */
public record ComponentVersionPage(List<ComponentVersion> items, long total, int page, int size)
{
}
