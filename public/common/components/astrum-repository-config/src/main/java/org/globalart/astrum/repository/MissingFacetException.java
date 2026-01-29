
package org.globalart.astrum.repository;

/**
 * Thrown when requested {@link Facet} does not exist.
 *
 * @since 3.0
 */
public class MissingFacetException
    extends RuntimeException
{
  public MissingFacetException(final Repository repository, final Class<? extends Facet> type) {
    super(String.format("No facet of type %s attached to repository %s", type.getSimpleName(), repository.getName()));
  }
}
