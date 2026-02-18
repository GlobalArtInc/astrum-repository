
package org.globalart.astrum.repository.cleanup;

public interface CleanupFeatureCheck
{
  boolean isSQLCleanupSupported();

  boolean isRetainSupported(String formatName);

  boolean isProVersion();

  boolean isPostgres();
}
