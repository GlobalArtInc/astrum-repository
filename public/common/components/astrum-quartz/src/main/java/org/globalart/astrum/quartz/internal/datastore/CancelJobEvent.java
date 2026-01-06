
package org.globalart.astrum.quartz.internal.datastore;

import org.globalart.astrum.common.event.EventWithSource;

public class CancelJobEvent
    extends EventWithSource
{
  private String id;

  private boolean mayInterruptIfRunning;

  CancelJobEvent() {
    // deserialization
  }

  public CancelJobEvent(final String id, final boolean mayInterruptIfRunning) {
    this.id = id;
    this.mayInterruptIfRunning = mayInterruptIfRunning;
  }

  public String getId() {
    return id;
  }

  public boolean isMayInterruptIfRunning() {
    return mayInterruptIfRunning;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public void setMayInterruptIfRunning(final boolean mayInterruptIfRunning) {
    this.mayInterruptIfRunning = mayInterruptIfRunning;
  }
}
