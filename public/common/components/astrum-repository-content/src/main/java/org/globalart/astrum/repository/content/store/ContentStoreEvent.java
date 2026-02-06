
package org.globalart.astrum.repository.content.store;

import java.util.Optional;
import java.util.function.Supplier;

import javax.annotation.Nullable;

import org.globalart.astrum.common.event.Event;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * Base event emitted by content stores.
 *
 * @since 3.26
 */
public class ContentStoreEvent
    implements Event
{
  final int contentRepositoryId;

  private Supplier<Optional<Repository>> repositorySupplier;

  protected ContentStoreEvent(final int contentRepositoryId) {
    this.contentRepositoryId = contentRepositoryId;
  }

  @Nullable
  public String getFormat() {
    return getRepository().map(Repository::getFormat).map(Format::getValue).orElse(null);
  }

  /**
   * May provide the Repository, repeatedly calling this method may provide different results.
   */
  public Optional<Repository> getRepository() {
    checkState(this.repositorySupplier != null, "Repository supplier has not been set");
    return repositorySupplier.get();
  }

  void setRepositorySupplier(final Supplier<Optional<Repository>> repositorySupplier) {
    checkState(this.repositorySupplier == null, "Repository supplier is already set");
    this.repositorySupplier = checkNotNull(repositorySupplier);
  }

  @Override
  public String toString() {
    Optional<Repository> repository = repositorySupplier == null ? null : repositorySupplier.get();
    return "ContentStoreEvent{" +
        "contentRepositoryId=" + contentRepositoryId +
        ", repository=" + repository +
        '}';
  }
}
