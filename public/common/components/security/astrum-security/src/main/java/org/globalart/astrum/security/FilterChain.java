
package org.globalart.astrum.security;

import java.util.stream.Collectors;
import java.util.stream.Stream;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A Shiro filter chain (mapping between a path pattern and a filter expression).
 *
 * @since 2.5
 */
public class FilterChain
{
  private final String pathPattern;

  private final String filterExpression;

  public FilterChain(final String pathPattern, final String... filterExpressions) {
    this.pathPattern = checkNotNull(pathPattern);
    this.filterExpression = Stream.of(checkNotNull(filterExpressions))
        .collect(Collectors.joining(","));
  }

  public String getPathPattern() {
    return pathPattern;
  }

  public String getFilterExpression() {
    return filterExpression;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "pathPattern='" + pathPattern + '\'' +
        ", filterExpression='" + filterExpression + '\'' +
        '}';
  }
}
