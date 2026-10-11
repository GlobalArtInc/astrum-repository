package org.globalart.astrum.repository.content.downloads;

import java.time.LocalDate;
import javax.annotation.Nullable;

/**
 * Downloads of the assets of one content repository, summed up by {@link AssetDownloadCountStore} or
 * {@link AssetDownloadDailyStore}: over all time, over a span of days, or on one day.
 */
public class RepositoryDownloads
{
  private String format;

  private int repositoryId;

  @Nullable
  private LocalDate downloadDay;

  private long downloadCount;

  private long componentCount;

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

  /**
   * The day of the downloads when they are summed up per day, else {@code null}.
   */
  @Nullable
  public LocalDate getDownloadDay() {
    return downloadDay;
  }

  public void setDownloadDay(@Nullable final LocalDate downloadDay) {
    this.downloadDay = downloadDay;
  }

  public long getDownloadCount() {
    return downloadCount;
  }

  public void setDownloadCount(final long downloadCount) {
    this.downloadCount = downloadCount;
  }

  /**
   * How many different components were downloaded, where the query counts them, else {@code 0}.
   */
  public long getComponentCount() {
    return componentCount;
  }

  public void setComponentCount(final long componentCount) {
    this.componentCount = componentCount;
  }

  @Override
  public String toString() {
    return "RepositoryDownloads{" +
        "format='" + format + '\'' +
        ", repositoryId=" + repositoryId +
        ", downloadDay=" + downloadDay +
        ", downloadCount=" + downloadCount +
        ", componentCount=" + componentCount +
        '}';
  }
}
