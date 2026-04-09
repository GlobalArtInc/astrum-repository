
package org.globalart.astrum.repository.helm.internal.createindex;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event class for use when invalidating Helm index.yaml file
 *
 * @since 3.28
 */
public class HelmIndexInvalidationEvent
{
  private final String repositoryName;

  private final boolean waitBeforeRebuild;

  public HelmIndexInvalidationEvent(final String repositoryName,
                                    final boolean waitBeforeRebuild) {
    this.repositoryName = checkNotNull(repositoryName);
    this.waitBeforeRebuild = waitBeforeRebuild;
  }

  /**
   * The helm repository that requires its index invalidated and rebuilt.
   */
  public String getRepositoryName() {
    return repositoryName;
  }

  /**
   * If set then force createIndex to wait before generating to prevent multiple re-writes being scheduled one after
   * another
   */
  public boolean isWaitBeforeRebuild() {
    return waitBeforeRebuild;
  }
}
