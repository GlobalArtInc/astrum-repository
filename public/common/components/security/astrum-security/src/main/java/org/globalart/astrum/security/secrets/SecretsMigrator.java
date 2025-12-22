
package org.globalart.astrum.security.secrets;

import org.globalart.astrum.crypto.secrets.Secret;

/**
 * Provides logic to migrate secrets from legacy encryption to encryption backed by secrets table.
 * Implementors must know if a secret was already migrated to avoid re-encrypting it.
 */
public interface SecretsMigrator
{
  void migrate() throws SecretMigrationException;

  /**
   * @return {@code true} if the secret is a simple wrapper around an encrypted string
   */
  default boolean isLegacyEncryptedString(final Secret secret) {
    return !isPersistedSecret(secret);
  }

  /**
   * @return {@code true} if the provided secret is a persisted secret
   */
  default boolean isPersistedSecret(final Secret secret) {
    return secret.getId().startsWith("_");
  }
}
