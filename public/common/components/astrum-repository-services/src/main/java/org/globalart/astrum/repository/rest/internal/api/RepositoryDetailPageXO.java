
package org.globalart.astrum.repository.rest.internal.api;

import java.util.List;

/**
 * Paginated response for repository details with server-side filtering support.
 * Enables enterprise-scale browse functionality with efficient data transfer.
 */
public class RepositoryDetailPageXO
{
  private final List<RepositoryDetailXO> data;

  private final int totalCount;

  private final int page;

  private final int pageSize;

  public RepositoryDetailPageXO(
      final List<RepositoryDetailXO> data,
      final int totalCount,
      final int page,
      final int pageSize)
  {
    this.data = data;
    this.totalCount = totalCount;
    this.page = page;
    this.pageSize = pageSize;
  }

  public List<RepositoryDetailXO> getData() {
    return data;
  }

  public int getTotalCount() {
    return totalCount;
  }

  public int getPage() {
    return page;
  }

  public int getPageSize() {
    return pageSize;
  }

  public int getTotalPages() {
    return (int) Math.ceil((double) totalCount / pageSize);
  }

  public boolean hasNextPage() {
    return page < getTotalPages();
  }

  public boolean hasPreviousPage() {
    return page > 1;
  }
}
