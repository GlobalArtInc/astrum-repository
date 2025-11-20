
package org.globalart.astrum.datastore.api;

/**
 * Thrown when an insert or update violates a foreign key constraint (SQLState 23503).
 */
public class ForeignKeyViolationException
    extends DataAccessException
{
  private static final long serialVersionUID = 1L;

  public static final String SQL_STATE = "23503";

  public ForeignKeyViolationException(final Throwable cause) {
    super("Foreign key violation", cause);
  }
}
