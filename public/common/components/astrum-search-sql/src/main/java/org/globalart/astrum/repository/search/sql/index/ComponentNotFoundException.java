
package org.globalart.astrum.repository.search.sql.index;

/**
 * Exception thrown when the component could not be found when during an attempt to refresh a component record in the
 * search table.
 *
 * @see SqlSearchIndexService
 * @since 3.43
 */
public class ComponentNotFoundException
    extends RuntimeException
{
  public ComponentNotFoundException(final String message) {
    super(message);
  }
}
