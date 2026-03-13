
package org.globalart.astrum.repository.search.sql.index;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.search.sql.store.SearchStore;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.search.index.SearchUpdateService;
import org.globalart.astrum.repository.types.GroupType;

import org.springframework.context.annotation.Primary;

import static com.google.common.base.Preconditions.checkNotNull;

import org.springframework.stereotype.Component;

/**
 * Determines if a SQL table search index should be updated with common {@link SearchUpdateService} logic.
 */
@Component
@Primary
public class SqlSearchUpdateService
    implements SearchUpdateService
{
  private final SearchStore searchTableStore;

  @Autowired
  public SqlSearchUpdateService(final SearchStore searchTableStore) {
    this.searchTableStore = checkNotNull(searchTableStore);
  }

  @Override
  public boolean needsReindex(final Repository repository) {
    if (GroupType.NAME.equals(repository.getType().getValue())) {
      return false;
    }
    ContentFacet contentFacet = repository.facet(ContentFacet.class);
    Object indexOutdated = contentFacet.attributes().get(SEARCH_INDEX_OUTDATED);
    if (indexOutdated instanceof Boolean) {
      return (boolean) indexOutdated;
    }
    return searchTableStore.repositoryNeedsReindex(repository.getName());
  }

  @Override
  public void doneReindexing(final Repository repository) {
    ContentFacet contentFacet = repository.facet(ContentFacet.class);
    contentFacet.withoutAttribute(SEARCH_INDEX_OUTDATED);
  }
}
