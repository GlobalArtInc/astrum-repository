
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Indicates that a Blob store configuration has been created, updated or deleted.
 *
 * @since 3.38
 */
public class BlobStoreDistributedConfigurationEvent
    extends DistributedEventSupport
{
  public static final String NAME = "BlobStoreConfigurationEvent";

  private final String blobStoreName;

  @JsonCreator
  public BlobStoreDistributedConfigurationEvent(
      @JsonProperty("blobStoreName") final String blobStoreName,
      @JsonProperty("eventType") final EventType eventType)
  {
    super(eventType);
    this.blobStoreName = checkNotNull(blobStoreName);
  }

  public String getBlobStoreName() {
    return blobStoreName;
  }

  @Override
  public String toString() {
    return "BlobStoreConfigurationEvent{" +
        "blobStoreName='" + blobStoreName + '\'' +
        '}';
  }
}
