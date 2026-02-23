
package org.globalart.astrum.repository.search;

/**
 * It should be used to determine if we have to handle blank values from the {@code SearchFilter}.
 */
public interface BlankValueSearchQueryFilter
{
  /**
   * Should we handle empty values for search attributes.
   *
   * @return {@code true} handle empty values, otherwise {@code false}.
   */
  boolean shouldHandleBlankValue();
}
