
package org.globalart.astrum.repository.search.sql.store;

import org.globalart.astrum.repository.search.sql.query.SearchConditionFactory;

/**
 * H2 runner for the component-versions queries, alongside
 * {@code PostgresComponentVersionsSearchTableDAOTest}. H2 is the default store for self-hosted
 * deployments, and these queries use constructs whose portability is not self-evident —
 * STRING_AGG with an ordered DISTINCT aggregate, LIKE ... ESCAPE, and COUNT(DISTINCT ...) — so
 * both engines need to execute them. Mirrors the {@link SearchTableDAOTest} /
 * {@code PostgresSearchTableDAOTest} pairing.
 */
public class ComponentVersionsSearchTableDAOTest
    extends ComponentVersionsSearchTableDAOTestSupport
{
  @Override
  protected SearchConditionFactory createSearchConditionFactory() {
    return SearchConditionFactories.forSession(sessionRule);
  }
}
