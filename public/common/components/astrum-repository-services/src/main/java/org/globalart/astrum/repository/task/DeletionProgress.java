
package org.globalart.astrum.repository.task;

/**
 * Tracks deletion progress.
 *
 * @since 3.25
 */
public class DeletionProgress
{
  private long componentCount = 0L;

  private long assetCount = 0L;

  private boolean failed;

  private int attempts = 0;

  private int retryLimit = 0;

  public DeletionProgress() {
    // use defaults
  }

  public DeletionProgress(final int retryLimit) {
    this.retryLimit = retryLimit;
  }

  public long getComponentCount() {
    return componentCount;
  }

  public void addComponentCount(final long count) {
    this.componentCount += count;
  }

  public long getAssetCount() {
    return assetCount;
  }

  public void addAssetCount(final long count) {
    this.assetCount += count;
  }

  public boolean isFailed() {
    return failed;
  }

  public void setFailed(final boolean completed) {
    this.failed = completed;
  }

  public int getAttempts() {
    return attempts;
  }

  public void setAttempts(final int attempts) {
    this.attempts = attempts;
  }

  public void update(final DeletionProgress progress) {
    failed = progress.isFailed();
    componentCount += progress.getComponentCount();
    assetCount += progress.getAssetCount();
    if (progress.isFailed()) {
      attempts++;
    }
  }

  public boolean isFinished() {
    return !isFailed() || getAttempts() >= retryLimit;
  }
}
