
package org.globalart.astrum.repository.proxy;

public class ProxyRequestEvent
{
  private final String format;

  private final boolean isReplication;

  public ProxyRequestEvent(final String format, final boolean isReplication) {
    this.format = format;
    this.isReplication = isReplication;
  }

  public String getFormat() {
    return format;
  }

  public boolean isReplication() {
    return isReplication;
  }
}
