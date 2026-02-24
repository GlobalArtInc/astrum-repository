
package org.globalart.astrum.repository.search.query;

import java.util.Set;

import org.globalart.astrum.repository.search.ComponentSearchResult;

/**
 *
 *
 * @since 3.14
 */
public interface SearchResultComponentGenerator
{
  ComponentSearchResult from(ComponentSearchResult hit, Set<String> componentIdSet);
}
