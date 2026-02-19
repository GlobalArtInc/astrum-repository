
package org.globalart.astrum.repository.rest;

/**
 * Provide a listing of all {@link SearchMapping}s that have been contributed via {@link SearchMappings}.
 *
 * @since 3.7
 */
public interface SearchMappingsService
{
  /**
   * Get all {@link SearchMapping}s.
   */
  Iterable<SearchMapping> getAllMappings();
}
