
package org.globalart.astrum.repository.search.sql.query;

import java.util.Arrays;
import java.util.Optional;

import org.globalart.astrum.repository.search.ComponentVersionSortKeys;

/**
 * The sort keys the version-browsing API accepts, and the SQL each maps to.
 *
 * <p>
 * The keys are defined canonically in {@link ComponentVersionSortKeys}. This enum maps
 * from those canonical keys to the SQL expressions interpolated into SearchTableDAO.xml.
 * Caller-supplied text must never reach that interpolation.
 */
public enum ComponentVersionSortField
{
  VERSION(ComponentVersionSortKeys.VERSION, "cs.normalised_version"),

  LAST_UPDATED(ComponentVersionSortKeys.LAST_UPDATED, "MAX(cs.last_modified)"),

  REPOSITORIES(ComponentVersionSortKeys.REPOSITORIES, "COUNT(DISTINCT cs.search_repository_name)");

  private final String key;

  private final String sqlExpression;

  ComponentVersionSortField(final String key, final String sqlExpression) {
    this.key = key;
    this.sqlExpression = sqlExpression;
  }

  public String key() {
    return key;
  }

  public String sqlExpression() {
    return sqlExpression;
  }

  public static Optional<ComponentVersionSortField> fromKey(final String key) {
    return Arrays.stream(values()).filter(field -> field.key.equals(key)).findFirst();
  }
}
