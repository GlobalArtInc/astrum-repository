
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.common.log.SupportZipGeneratorRequest;
import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Indicates that 'Support Zip' should be created in some node.
 * Contains the 'request' object which holds all properties for 'support zip' creation and nodeID of target node
 */
public class SupportZipGenerationEvent
    extends DistributedEventSupport
{
  public static final String NAME = "SupportZipGenerationEvent";

  private final String recipientNodeId;

  private final SupportZipGeneratorRequest request;

  @JsonCreator
  public SupportZipGenerationEvent(
      @JsonProperty("recipientNodeId") final String recipientNodeId,
      @JsonProperty("request") final SupportZipGeneratorRequest request)
  {
    super(EventType.CREATED);
    this.recipientNodeId = checkNotNull(recipientNodeId);
    this.request = checkNotNull(request);
  }

  public String getRecipientNodeId() {
    return recipientNodeId;
  }

  public SupportZipGeneratorRequest getRequest() {
    return request;
  }

  @Override
  public String toString() {
    return "SupportZipGenerationEvent{" +
        "recipientNodeId='" + recipientNodeId + '\'' +
        ", request=" + request +
        '}';
  }
}
