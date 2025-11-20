
package org.globalart.astrum.datastore.api;

public class SerializedAccessException
    extends DataAccessException
{
  private static final long serialVersionUID = 98739582308995723L;

  public static final String SQL_STATE = "40001";

  public SerializedAccessException(final Throwable cause) {
    super("Duplicate key", cause);
  }
}
