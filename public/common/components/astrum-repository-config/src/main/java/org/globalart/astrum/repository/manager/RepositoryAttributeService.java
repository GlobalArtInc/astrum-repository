
package org.globalart.astrum.repository.manager;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;

/**
 * Service interface for setting and getting repository attributes.
 * This interface allows modules to depend on attribute functionality
 * without creating circular dependencies.
 *
 * @since 3.84
 */
public interface RepositoryAttributeService
{
  /**
   * Sets a repository attribute using the best available method
   * (ContentFacet, AttributeStorage service, etc.)
   *
   * @param repository the repository (as Object to avoid dependencies)
   * @param key the attribute key
   * @param value the attribute value
   * @return true if the attribute was successfully set
   */
  boolean setRepositoryAttribute(final Repository repository, final String key, final Object value);

  /**
   * Gets a repository attribute using the best available method.
   *
   * @param repository the repository (as Object to avoid dependencies)
   * @param key the attribute key
   * @param defaultValue the default value if attribute is not found
   * @return the attribute value or default value
   */
  @Nullable
  String getRepositoryAttribute(final Repository repository, final String key, final @Nullable String defaultValue);
}
