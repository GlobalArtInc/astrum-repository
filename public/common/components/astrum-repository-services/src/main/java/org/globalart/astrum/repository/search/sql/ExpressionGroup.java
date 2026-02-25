
package org.globalart.astrum.repository.search.sql;

import org.globalart.astrum.repository.search.sql.query.syntax.Expression;

/**
 * It is a container class for the three types of expression filters that can be applied to a query;
 * component filters, asset filters, and content selector filters.
 */
public class ExpressionGroup
{
  private final Expression componentFilters;

  private final Expression assetFilters;

  public ExpressionGroup(
      final Expression componentFilters,
      final Expression assetFilters)
  {
    this.componentFilters = componentFilters;
    this.assetFilters = assetFilters;
  }

  public Expression getComponentFilters() {
    return componentFilters;
  }

  public Expression getAssetFilters() {
    return assetFilters;
  }

  @Override
  public String toString() {
    return String.format(
        "ExpressionGroup{componentFilters=%s, assetFilters=%s}",
        componentFilters, assetFilters);
  }
}
