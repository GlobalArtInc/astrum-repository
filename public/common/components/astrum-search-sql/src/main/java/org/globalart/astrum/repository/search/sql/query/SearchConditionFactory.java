
package org.globalart.astrum.repository.search.sql.query;

import org.globalart.astrum.repository.search.sql.store.SearchStore;

import org.globalart.astrum.repository.search.sql.ExpressionGroup;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;

/**
 * Factory which converts the database agnostic {@link Expression} into a database specific
 * {@link SqlSearchQueryCondition} which can be passed to {@link SearchStore}
 */
public interface SearchConditionFactory
{
  SqlSearchQueryConditionGroup build(ExpressionGroup expression);
}
