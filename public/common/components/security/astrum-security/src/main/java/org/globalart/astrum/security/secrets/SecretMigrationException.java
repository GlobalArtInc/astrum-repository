
package org.globalart.astrum.security.secrets;

/**
 * Exception to identify errors related to secret migration
 */
public class SecretMigrationException
    extends RuntimeException
{
  public SecretMigrationException(final String message) {
    super(message);
  }

  public SecretMigrationException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
