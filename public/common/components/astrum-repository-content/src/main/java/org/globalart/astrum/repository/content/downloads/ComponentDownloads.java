package org.globalart.astrum.repository.content.downloads;

/**
 * Downloads of the assets of one component, summed up by {@link AssetDownloadCountStore} or
 * {@link AssetDownloadDailyStore}.
 */
public class ComponentDownloads
{
  private String format;

  private int repositoryId;

  private int componentId;

  private long downloadCount;

  public ComponentDownloads() {
    // for MyBatis
  }

  public ComponentDownloads(
      final String format,
      final int repositoryId,
      final int componentId,
      final long downloadCount)
  {
    this.format = format;
    this.repositoryId = repositoryId;
    this.componentId = componentId;
    this.downloadCount = downloadCount;
  }

  public String getFormat() {
    return format;
  }

  public void setFormat(final String format) {
    this.format = format;
  }

  public int getRepositoryId() {
    return repositoryId;
  }

  public void setRepositoryId(final int repositoryId) {
    this.repositoryId = repositoryId;
  }

  public int getComponentId() {
    return componentId;
  }

  public void setComponentId(final int componentId) {
    this.componentId = componentId;
  }

  public long getDownloadCount() {
    return downloadCount;
  }

  public void setDownloadCount(final long downloadCount) {
    this.downloadCount = downloadCount;
  }

  @Override
  public String toString() {
    return "ComponentDownloads{" +
        "format='" + format + '\'' +
        ", repositoryId=" + repositoryId +
        ", componentId=" + componentId +
        ", downloadCount=" + downloadCount +
        '}';
  }
}
