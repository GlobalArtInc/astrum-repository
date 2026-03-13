
package org.globalart.astrum.repository.search.sql.query.h2;

import org.globalart.astrum.repository.search.sql.ExpressionGroup;
import org.globalart.astrum.repository.search.sql.query.SearchConditionFactory;
import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryCondition;
import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryConditionGroup;
import org.globalart.astrum.repository.search.sql.query.h2.H2SearchConditionBuilder.ConditionType;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link SearchConditionFactory} which produces queries for use with H2 database.
 * Uses LIKE queries instead of PostgreSQL's full-text search.
 * Instantiated by SearchConditionFactoryImpl when H2 is detected at runtime.
 */
@Lazy
@Qualifier("h2")
@Component
public class H2SearchConditionFactory
    implements SearchConditionFactory
{
  private final H2SearchDB db;

  @Autowired
  public H2SearchConditionFactory(final H2SearchDB db) {
    this.db = checkNotNull(db);
  }

  @Override
  public SqlSearchQueryConditionGroup build(final ExpressionGroup expressionGroup) {
    final Expression componentFilters = expressionGroup.getComponentFilters();
    SqlSearchQueryCondition componentQueryCondition = null;
    if (componentFilters != null) {
      final H2FulltextSearchConditionBuilder componentFilterBuilder =
          new H2FulltextSearchConditionBuilder(db);
      componentQueryCondition = componentFilterBuilder.build(componentFilters);
    }

    final Expression assetFilters = expressionGroup.getAssetFilters();
    SqlSearchQueryCondition assetQueryCondition = null;
    if (assetFilters != null) {
      final H2FulltextSearchConditionBuilder assetFilterBuilder =
          new H2FulltextSearchConditionBuilder(db, ConditionType.ASSET_FILTER);
      assetQueryCondition = assetFilterBuilder.build(assetFilters);
    }

    return new SqlSearchQueryConditionGroup(componentQueryCondition, assetQueryCondition);
  }
}
