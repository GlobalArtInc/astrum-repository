
package org.globalart.astrum.repository.content.store;

import java.util.Optional;
import java.util.function.Supplier;

import org.globalart.astrum.repository.Repository;

/**
 * Test utility that provides access to {@link ContentStoreEvent#setRepositorySupplier},
 * which is package-private. Tests in other packages can use this instead of reflection.
 */
public final class ContentStoreEventSetter
{
  private ContentStoreEventSetter() {
    // utility class
  }

  public static void setRepositorySupplier(
      final ContentStoreEvent event,
      final Supplier<Optional<Repository>> repositorySupplier)
  {
    event.setRepositorySupplier(repositorySupplier);
  }
}
