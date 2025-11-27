
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

/**
 * Indicates that the OIDC/OAuth2 configuration changed, so all nodes must invalidate their in-memory cache.
 */
public class OidcConfigurationDistributedEvent
    extends DistributedEventSupport
{
  public static final String NAME = "OidcConfigurationDistributedEvent";

  public OidcConfigurationDistributedEvent() {
    super(EventType.UPDATED);
  }

  @Override
  public String toString() {
    return NAME + "{" +
        "eventType='" + getEventType() + '\'' +
        "}";
  }
}
