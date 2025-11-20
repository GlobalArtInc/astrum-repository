
package org.globalart.astrum.datastore.api;

/**
 * Thrown when inserting/updating would break a unique key constraint.
 *
 * @since 3.26
 */
public class DuplicateKeyException
    extends DataAccessException
{
  private static final long serialVersionUID = 98739582308995723L;

  public static final String SQL_STATE = "23505";

  public DuplicateKeyException(final Throwable cause) {
    super("Duplicate key", cause);
  }
}
