package org.globalart.astrum.repository.content.usage;

/**
 * What the content of one content repository of a format takes up, as read by {@link UsageStore}.
 *
 * A component counts as unused when none of its assets was downloaded since a given time and none was added since
 * then either, the way cleanup policies judge the last download; it counts as never downloaded when none of its assets
 * has a last downloaded time.
 */
public class RepositoryContentUsage
{
  private int repositoryId;

  private long components;

  private long assets;

  private long bytes;

  private long unusedComponents;

  private long unusedBytes;

  private long neverDownloadedComponents;

  private long neverDownloadedBytes;

  public int getRepositoryId() {
    return repositoryId;
  }

  public void setRepositoryId(final int repositoryId) {
    this.repositoryId = repositoryId;
  }

  public long getComponents() {
    return components;
  }

  public void setComponents(final long components) {
    this.components = components;
  }

  public long getAssets() {
    return assets;
  }

  public void setAssets(final long assets) {
    this.assets = assets;
  }

  public long getBytes() {
    return bytes;
  }

  public void setBytes(final long bytes) {
    this.bytes = bytes;
  }

  public long getUnusedComponents() {
    return unusedComponents;
  }

  public void setUnusedComponents(final long unusedComponents) {
    this.unusedComponents = unusedComponents;
  }

  public long getUnusedBytes() {
    return unusedBytes;
  }

  public void setUnusedBytes(final long unusedBytes) {
    this.unusedBytes = unusedBytes;
  }

  public long getNeverDownloadedComponents() {
    return neverDownloadedComponents;
  }

  public void setNeverDownloadedComponents(final long neverDownloadedComponents) {
    this.neverDownloadedComponents = neverDownloadedComponents;
  }

  public long getNeverDownloadedBytes() {
    return neverDownloadedBytes;
  }

  public void setNeverDownloadedBytes(final long neverDownloadedBytes) {
    this.neverDownloadedBytes = neverDownloadedBytes;
  }

  @Override
  public String toString() {
    return "RepositoryContentUsage{" +
        "repositoryId=" + repositoryId +
        ", components=" + components +
        ", assets=" + assets +
        ", bytes=" + bytes +
        ", unusedComponents=" + unusedComponents +
        ", unusedBytes=" + unusedBytes +
        ", neverDownloadedComponents=" + neverDownloadedComponents +
        ", neverDownloadedBytes=" + neverDownloadedBytes +
        '}';
  }
}
