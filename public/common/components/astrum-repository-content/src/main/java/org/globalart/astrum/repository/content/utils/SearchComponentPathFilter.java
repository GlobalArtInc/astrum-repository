
package org.globalart.astrum.repository.content.utils;

/**
 * Filters search_component table path extensions by repository type
 *
 */
public interface SearchComponentPathFilter
{

  /**
   * Evaluates whether the search_component table path needs to be filtered based on the most common extension paths
   * Implementations are format specific.
   */
  boolean shouldFilterPathExtension(String path);
}
