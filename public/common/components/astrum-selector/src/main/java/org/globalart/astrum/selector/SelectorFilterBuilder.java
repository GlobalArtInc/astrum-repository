
package org.globalart.astrum.selector;

import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

/**
 * Builds a SQL filter from a list of content selectors.
 */
public interface SelectorFilterBuilder
{
  /**
   * Builds a SQL filter for a list of content selectors.
   *
   * @param format format of the repository that the filter will run on
   * @param pathAlias alias that can be used to refer to the path inside the SQL
   * @param selectors set of content selectors to build filter from
   * @param filterParameters "out" parameter in which filter parameter keys and values are placed
   * @return filter query ready to be appended to a SQL statement
   */
  @Nullable
  String buildFilter(
      final String format,
      final String pathAlias,
      final List<SelectorConfiguration> selectors,
      final Map<String, Object> filterParameters);
}
