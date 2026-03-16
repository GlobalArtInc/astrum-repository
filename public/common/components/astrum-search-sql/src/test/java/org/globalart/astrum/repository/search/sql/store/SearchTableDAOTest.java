
package org.globalart.astrum.repository.search.sql.store;

import org.globalart.astrum.repository.search.sql.query.SearchConditionFactory;

public class SearchTableDAOTest
    extends SearchTableDAOTestSupport
{
  @Override
  protected SearchConditionFactory createSearchConditionFactory() {
    return SearchConditionFactories.forSession(sessionRule);
  }
}
