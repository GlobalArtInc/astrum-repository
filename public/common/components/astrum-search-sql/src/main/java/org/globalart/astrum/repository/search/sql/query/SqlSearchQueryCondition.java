
package org.globalart.astrum.repository.search.sql.query;

import java.util.Map;

import static java.util.Collections.unmodifiableMap;

/**
 * Encapsulates a sql query condition's format string e.g. <code>field IN (#{field0}, #{field1}, #{field2})</code> and
 * its associated values (which in the example given will be a <code>Map&lt;String, String&gt;</code> of named
 * parameters and their associated values).
 *
 * @param Sql condition format string e.g. <code>field IN (#{field0}, #{field1}, #{field2})</code>
 * @param Mapped of named values to apply to the Sql condition format string.
 *
 * @since 3.38
 */
public record SqlSearchQueryCondition(String sqlFilter, Map<String, Object> parameters)
{
  public String getSqlFilter() {
    return sqlFilter;
  }

  public Map<String, Object> getParameters() {
    return unmodifiableMap(parameters);
  }
}
