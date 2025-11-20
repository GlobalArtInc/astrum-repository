
package org.globalart.astrum.datastore.api;

/**
 * Base exception for any issues relating to data access/updates.
 *
 * @since 3.26
 */
public class DataAccessException
    extends RuntimeException
{
  private static final long serialVersionUID = 8359662557565269872L;

  public DataAccessException(final String message) {
    super(message);
  }

  public DataAccessException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
