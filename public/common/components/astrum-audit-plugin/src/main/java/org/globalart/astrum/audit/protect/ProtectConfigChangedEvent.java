
package org.globalart.astrum.audit.protect;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Posted on the event bus when Protect-related configuration changes (firewall, health check, cleanup task).
 */
public class ProtectConfigChangedEvent
{
  private final String domain;

  private final String type;

  private final String context;

  private final String fromValue;

  private final String toValue;

  public ProtectConfigChangedEvent(
      final String domain,
      final String type,
      final String context,
      final String fromValue,
      final String toValue)
  {
    this.domain = checkNotNull(domain);
    this.type = checkNotNull(type);
    this.context = checkNotNull(context);
    this.fromValue = checkNotNull(fromValue);
    this.toValue = checkNotNull(toValue);
  }

  public String getDomain() {
    return domain;
  }

  public String getType() {
    return type;
  }

  public String getContext() {
    return context;
  }

  public String getFromValue() {
    return fromValue;
  }

  public String getToValue() {
    return toValue;
  }
}
