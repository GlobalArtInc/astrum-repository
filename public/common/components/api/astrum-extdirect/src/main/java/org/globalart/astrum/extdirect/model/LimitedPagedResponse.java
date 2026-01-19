
package org.globalart.astrum.extdirect.model;

import java.util.Collection;

/**
 * Ext.Direct paged response with a limit on the total.
 *
 * @since 3.1
 */
public class LimitedPagedResponse<T>
    extends PagedResponse<T>
{
  private long unlimitedTotal;

  private boolean limited;

  private boolean timedOut;

  public LimitedPagedResponse(long limit, long total, Collection<T> data) {
    super(Math.min(limit, total), data);
    this.unlimitedTotal = total;
    this.limited = total != getTotal();
  }

  public LimitedPagedResponse(long limit, long total, Collection<T> data, boolean timedOut) {
    this(limit, total, data);
    this.timedOut = timedOut;
  }

  public long getUnlimitedTotal() {
    return unlimitedTotal;
  }

  public boolean isLimited() {
    return limited;
  }

  public boolean isTimedOut() {
    return timedOut;
  }
}
