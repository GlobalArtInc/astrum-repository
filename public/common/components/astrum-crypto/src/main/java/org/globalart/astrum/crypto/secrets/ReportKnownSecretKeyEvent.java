
package org.globalart.astrum.crypto.secrets;

import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An event used to indicate that nodes should report whether the specified
 * encryption key for secrets is known.
 */
public class ReportKnownSecretKeyEvent
    extends EventWithSource
{
  private final String keyId;

  @JsonCreator
  public ReportKnownSecretKeyEvent(@JsonProperty("keyId") final String keyId) {
    this.keyId = checkNotNull(keyId);
  }

  public String getKeyId() {
    return keyId;
  }

}
