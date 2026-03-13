package org.globalart.astrum.repository.search.sql.query.postgres;

import org.globalart.astrum.repository.search.sql.ExpressionGroup;
import org.globalart.astrum.repository.search.sql.query.SearchConditionFactory;
import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryCondition;
import org.globalart.astrum.repository.search.sql.query.SqlSearchQueryConditionGroup;
import org.globalart.astrum.repository.search.sql.query.postgres.PostgresSearchConditionBuilder.ConditionType;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Lazy
@Qualifier("postgres")
@Component
public class PostgresSearchConditionFactory
    implements SearchConditionFactory
{
  private final PostgresSearchDB db;

  @Autowired
  public PostgresSearchConditionFactory(final PostgresSearchDB db) {
    this.db = checkNotNull(db);
  }

  @Override
  public SqlSearchQueryConditionGroup build(final ExpressionGroup expressionGroup) {
    Expression componentFilters = expressionGroup.getComponentFilters();
    SqlSearchQueryCondition componentQueryCondition = null;
    if (componentFilters != null) {
      componentQueryCondition = new PostgresFulltextSearchConditionBuilder(db).build(componentFilters);
    }

    Expression assetFilters = expressionGroup.getAssetFilters();
    SqlSearchQueryCondition assetQueryCondition = null;
    if (assetFilters != null) {
      assetQueryCondition =
          new PostgresFulltextSearchConditionBuilder(db, ConditionType.ASSET_FILTER).build(assetFilters);
    }

    return new SqlSearchQueryConditionGroup(componentQueryCondition, assetQueryCondition);
  }
}
