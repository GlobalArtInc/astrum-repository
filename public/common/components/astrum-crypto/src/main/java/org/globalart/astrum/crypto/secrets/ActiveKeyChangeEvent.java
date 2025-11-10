
package org.globalart.astrum.crypto.secrets;

import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An event fired when the active secret encryption key is changed.
 */
public class ActiveKeyChangeEvent
    extends EventWithSource
{
  private final String newKeyId;

  private final String previousKeyId;

  private final String userId;

  @JsonCreator
  public ActiveKeyChangeEvent(
      @JsonProperty("newKeyId") final String newKeyId,
      @JsonProperty("previousKeyId") final String previousKeyId,
      @JsonProperty("userId") final String userId)
  {
    this.newKeyId = checkNotNull(newKeyId);
    this.previousKeyId = previousKeyId;
    this.userId = userId;
  }

  public String getNewKeyId() {
    return newKeyId;
  }

  public String getPreviousKeyId() {
    return previousKeyId;
  }

  public String getUserId() {
    return userId;
  }
}
