package org.globalart.astrum.cleanup.preview;

/**
 * What a cleanup policy would delete from one repository: the matching components, their assets and the total size
 * of the assets' blobs in bytes. When {@link #isComplete()} is false the calculation stopped at its time limit and the
 * numbers are a lower bound.
 */
public class CleanupPreviewSummaryXO
{
  private final String repository;

  private final long componentCount;

  private final long assetCount;

  private final long totalSize;

  private final boolean complete;

  public CleanupPreviewSummaryXO(
      final String repository,
      final long componentCount,
      final long assetCount,
      final long totalSize,
      final boolean complete)
  {
    this.repository = repository;
    this.componentCount = componentCount;
    this.assetCount = assetCount;
    this.totalSize = totalSize;
    this.complete = complete;
  }

  public String getRepository() {
    return repository;
  }

  public long getComponentCount() {
    return componentCount;
  }

  public long getAssetCount() {
    return assetCount;
  }

  public long getTotalSize() {
    return totalSize;
  }

  public boolean isComplete() {
    return complete;
  }

  @Override
  public String toString() {
    return "CleanupPreviewSummaryXO(" +
        "repository:" + repository +
        ", componentCount:" + componentCount +
        ", assetCount:" + assetCount +
        ", totalSize:" + totalSize +
        ", complete:" + complete +
        ")";
  }
}
