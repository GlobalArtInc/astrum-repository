package org.globalart.astrum.repository.content.downloads;

import java.time.OffsetDateTime;
import java.util.Objects;
import javax.annotation.Nullable;

/**
 * Row of the {@code asset_download_count} table: how often an asset was downloaded and when it was downloaded last.
 *
 * Assets are identified by their format and internal id, since every format keeps its assets in a table of its own.
 * When written, {@link #getDownloadCount()} is the number of downloads to add to the stored count.
 */
public class AssetDownloadCountData
{
  private String format;

  private int assetId;

  private int repositoryId;

  @Nullable
  private Integer componentId;

  private long downloadCount;

  private OffsetDateTime lastDownloaded;

  public AssetDownloadCountData() {
    // for MyBatis
  }

  public AssetDownloadCountData(
      final String format,
      final int assetId,
      final int repositoryId,
      @Nullable final Integer componentId,
      final long downloadCount,
      final OffsetDateTime lastDownloaded)
  {
    this.format = format;
    this.assetId = assetId;
    this.repositoryId = repositoryId;
    this.componentId = componentId;
    this.downloadCount = downloadCount;
    this.lastDownloaded = lastDownloaded;
  }

  public String getFormat() {
    return format;
  }

  public void setFormat(final String format) {
    this.format = format;
  }

  public int getAssetId() {
    return assetId;
  }

  public void setAssetId(final int assetId) {
    this.assetId = assetId;
  }

  public int getRepositoryId() {
    return repositoryId;
  }

  public void setRepositoryId(final int repositoryId) {
    this.repositoryId = repositoryId;
  }

  @Nullable
  public Integer getComponentId() {
    return componentId;
  }

  public void setComponentId(@Nullable final Integer componentId) {
    this.componentId = componentId;
  }

  public long getDownloadCount() {
    return downloadCount;
  }

  public void setDownloadCount(final long downloadCount) {
    this.downloadCount = downloadCount;
  }

  public OffsetDateTime getLastDownloaded() {
    return lastDownloaded;
  }

  public void setLastDownloaded(final OffsetDateTime lastDownloaded) {
    this.lastDownloaded = lastDownloaded;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AssetDownloadCountData that = (AssetDownloadCountData) o;
    return assetId == that.assetId
        && repositoryId == that.repositoryId
        && downloadCount == that.downloadCount
        && Objects.equals(format, that.format)
        && Objects.equals(componentId, that.componentId)
        && Objects.equals(lastDownloaded, that.lastDownloaded);
  }

  @Override
  public int hashCode() {
    return Objects.hash(format, assetId, repositoryId, componentId, downloadCount, lastDownloaded);
  }

  @Override
  public String toString() {
    return "AssetDownloadCountData{" +
        "format='" + format + '\'' +
        ", assetId=" + assetId +
        ", repositoryId=" + repositoryId +
        ", componentId=" + componentId +
        ", downloadCount=" + downloadCount +
        ", lastDownloaded=" + lastDownloaded +
        '}';
  }
}
