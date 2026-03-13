
package org.globalart.astrum.repository.search.sql.query;

import java.util.Objects;
import java.util.Optional;

/**
 * Represents a group of SQL search conditions for components and assets.
 */
public class SqlSearchQueryConditionGroup
{
  private SqlSearchQueryCondition componentCondition;

  private SqlSearchQueryCondition assetCondition;

  public SqlSearchQueryConditionGroup(
      final SqlSearchQueryCondition componentQueryCondition,
      final SqlSearchQueryCondition assetQueryCondition)
  {
    this.componentCondition = componentQueryCondition;
    this.assetCondition = assetQueryCondition;
  }

  public SqlSearchQueryCondition getComponentCondition() {
    return componentCondition;
  }

  public Optional<SqlSearchQueryCondition> getAssetCondition() {
    return Optional.ofNullable(assetCondition);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    SqlSearchQueryConditionGroup that = (SqlSearchQueryConditionGroup) o;
    return Objects.equals(componentCondition, that.componentCondition) &&
        Objects.equals(assetCondition, that.assetCondition);
  }

  @Override
  public int hashCode() {
    return Objects.hash(componentCondition, assetCondition);
  }

  @Override
  public String toString() {
    return "SqlSearchQueryConditionGroup{" +
        "componentCondition=" + componentCondition +
        ", assetCondition=" + assetCondition +
        '}';
  }
}
