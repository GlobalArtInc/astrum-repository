
package org.globalart.astrum.repository.rest.api;

import org.joda.time.DateTime;

/**
 * Repository Metrics DTO
 */
public class RepositoryMetricsDTO
{
  public String repositoryName;

  public String repositoryFormat;

  public Long totalSize;

  public Long blobCount;

  public DateTime lastUpdated;

  public RepositoryMetricsDTO(
      String repositoryName,
      String repositoryFormat,
      Long totalSize,
      Long blobCount,
      DateTime lastUpdated)
  {
    this.repositoryName = repositoryName;
    this.repositoryFormat = repositoryFormat;
    this.totalSize = totalSize;
    this.blobCount = blobCount;
    this.lastUpdated = lastUpdated;
  }

  public RepositoryMetricsDTO(String repositoryName, Long totalSize) {
    this.repositoryName = repositoryName;
    this.totalSize = totalSize;
  }

  public String getName() {
    return repositoryName;
  }

  public Long getSize() {
    return totalSize;
  }

  public DateTime getLastUpdated() {
    return lastUpdated;
  }

  @Override
  public String toString() {
    return "RepositoryMetricsDTO{" +
        "repositoryName='" + repositoryName + '\'' +
        ", repositoryFormat='" + repositoryFormat + '\'' +
        ", totalSize=" + totalSize +
        ", blobCount=" + blobCount +
        ", lastUpdated=" + lastUpdated +
        '}';
  }
}
