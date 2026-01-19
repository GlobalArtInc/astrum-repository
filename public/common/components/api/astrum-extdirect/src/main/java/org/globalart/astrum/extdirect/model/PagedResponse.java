
package org.globalart.astrum.extdirect.model;

import java.util.Collection;

/**
 * Ext.Direct paged response.
 *
 * @since 3.0
 */
public class PagedResponse<T>
    extends Response<Collection<T>>
{
  private long total;

  public PagedResponse(long total, Collection<T> data) {
    super(true, data);
    this.total = total;
  }

  public long getTotal() {
    return total;
  }
}
