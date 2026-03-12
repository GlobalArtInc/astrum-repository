
package org.globalart.astrum.upgrade.datastore;

/**
 * An exception which occurs during a database migration
 */
public class UpgradeException
    extends Exception
{
  private static final long serialVersionUID = 1L;

  public UpgradeException(final String message, final Throwable cause) {
    super(message, cause);
  }

  public UpgradeException(final String message) {
    super(message);
  }
}
