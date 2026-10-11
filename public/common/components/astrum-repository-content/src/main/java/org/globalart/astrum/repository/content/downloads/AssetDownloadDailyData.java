package org.globalart.astrum.repository.content.downloads;

import java.time.LocalDate;
import java.util.Objects;

/**
 * Row of the {@code asset_download_daily} table: how often the assets of a component were downloaded on one day.
 *
 * Days are UTC days. Downloads of assets without a component are kept under the component id {@link #NO_COMPONENT}
 * of their repository, so that the rows of a repository add up to all of its downloads. When written,
 * {@link #getDownloadCount()} is the number of downloads to add to the stored count.
 */
public class AssetDownloadDailyData
{
  /**
   * Component id of the downloads of assets without a component; internal ids start at 1.
   */
  public static final int NO_COMPONENT = 0;

  private LocalDate downloadDay;

  private String format;

  private int repositoryId;

  private int componentId;

  private long downloadCount;

  public AssetDownloadDailyData() {
    // for MyBatis
  }

  public AssetDownloadDailyData(
      final LocalDate downloadDay,
      final String format,
      final int repositoryId,
      final int componentId,
      final long downloadCount)
  {
    this.downloadDay = downloadDay;
    this.format = format;
    this.repositoryId = repositoryId;
    this.componentId = componentId;
    this.downloadCount = downloadCount;
  }

  public LocalDate getDownloadDay() {
    return downloadDay;
  }

  public void setDownloadDay(final LocalDate downloadDay) {
    this.downloadDay = downloadDay;
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
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AssetDownloadDailyData that = (AssetDownloadDailyData) o;
    return repositoryId == that.repositoryId
        && componentId == that.componentId
        && downloadCount == that.downloadCount
        && Objects.equals(downloadDay, that.downloadDay)
        && Objects.equals(format, that.format);
  }

  @Override
  public int hashCode() {
    return Objects.hash(downloadDay, format, repositoryId, componentId, downloadCount);
  }

  @Override
  public String toString() {
    return "AssetDownloadDailyData{" +
        "downloadDay=" + downloadDay +
        ", format='" + format + '\'' +
        ", repositoryId=" + repositoryId +
        ", componentId=" + componentId +
        ", downloadCount=" + downloadCount +
        '}';
  }
}
