
package org.globalart.astrum.repository.content.search.upgrade;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.search.index.SearchUpdateService;
import org.globalart.astrum.repository.types.GroupType;
import org.springframework.stereotype.Component;

@Component
public class SearchUpdateServiceImpl
    implements SearchUpdateService
{
  @Override
  public boolean needsReindex(final Repository repository) {
    if (GroupType.NAME.equals(repository.getType().getValue())) {
      return false;
    }
    ContentFacet contentFacet = repository.facet(ContentFacet.class);
    Object indexOutdated = contentFacet.attributes().get(SEARCH_INDEX_OUTDATED);
    if (indexOutdated instanceof Boolean) {
      return (Boolean) indexOutdated;
    }
    return false;
  }

  @Override
  public void doneReindexing(final Repository repository) {
    ContentFacet contentFacet = repository.facet(ContentFacet.class);
    contentFacet.withoutAttribute(SEARCH_INDEX_OUTDATED);
  }
}
