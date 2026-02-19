
package org.globalart.astrum.repository.rest.api;

import java.util.List;

import io.swagger.v3.oas.annotations.media.Schema;

/**
 * One page of a component's distinct versions.
 */
@Schema(description = "One page of a component's distinct versions")
public class ComponentVersionsPageXO
{
  @Schema(description = "The versions on this page, ordered as requested")
  private List<ComponentVersionXO> items;

  @Schema(description = "Total number of distinct versions matching the query")
  private long total;

  @Schema(description = "Zero-based index of this page")
  private int page;

  @Schema(description = "Requested page size")
  private int size;

  public List<ComponentVersionXO> getItems() {
    return items;
  }

  public void setItems(final List<ComponentVersionXO> items) {
    this.items = items;
  }

  public long getTotal() {
    return total;
  }

  public void setTotal(final long total) {
    this.total = total;
  }

  public int getPage() {
    return page;
  }

  public void setPage(final int page) {
    this.page = page;
  }

  public int getSize() {
    return size;
  }

  public void setSize(final int size) {
    this.size = size;
  }
}
