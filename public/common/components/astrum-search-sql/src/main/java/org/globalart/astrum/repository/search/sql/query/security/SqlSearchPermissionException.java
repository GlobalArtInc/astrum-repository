
package org.globalart.astrum.repository.search.sql.query.security;

/**
 * Exception throw when a user does not have 'BROWSE' permissions nor content selectors for repositories being searched.
 */
public class SqlSearchPermissionException
    extends RuntimeException
{
  private static final long serialVersionUID = 656168831219901783L;

  public SqlSearchPermissionException(final String message) {
    super(message);
  }
}
