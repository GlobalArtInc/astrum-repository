
package org.globalart.astrum.distributed.event.service.api.common;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static org.globalart.astrum.distributed.event.service.api.EventType.UPDATED;

/**
 * Indicates that a user password has changed and authentication cache should be cleared.
 */
public class UserPasswordChangedDistributedEvent
    extends DistributedEventSupport
{
  public static final String NAME = "UserPasswordChangedDistributedEvent";

  private final String userId;

  private final boolean clearCache;

  @JsonCreator
  public UserPasswordChangedDistributedEvent(
      @JsonProperty("userId") final String userId,
      @JsonProperty("clearCache") final boolean clearCache)
  {
    super(UPDATED);
    this.userId = userId;
    this.clearCache = clearCache;
  }

  public String getUserId() {
    return userId;
  }

  public boolean isClearCache() {
    return clearCache;
  }
}
