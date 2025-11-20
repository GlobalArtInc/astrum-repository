
package org.globalart.astrum.datastore.api;

/**
 * Thrown when a named {@link DataStore} was not found.
 *
 * @since 3.19
 */
public class DataStoreNotFoundException
    extends DataAccessException
{
  private static final long serialVersionUID = -7516739829244823213L;

  public DataStoreNotFoundException(final String storeName) {
    super("Data store not found: '" + storeName + "'");
  }
}
