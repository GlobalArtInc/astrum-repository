
package org.globalart.astrum.audit.internal.rest;

import java.time.OffsetDateTime;
import java.util.Map;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Transfer object for audit event data exposed via REST API.
 */
public class AuditEventXO
{
  private final Long id;

  private final String domain;

  private final String type;

  private final String context;

  private final OffsetDateTime timestamp;

  private final String initiator;

  private final String nodeId;

  private final Map<String, Object> attributes;

  @JsonCreator
  public AuditEventXO(
      @JsonProperty("id") final Long id,
      @JsonProperty("domain") final String domain,
      @JsonProperty("type") final String type,
      @JsonProperty("context") final String context,
      @JsonProperty("timestamp") final OffsetDateTime timestamp,
      @JsonProperty("initiator") final String initiator,
      @JsonProperty("nodeId") final String nodeId,
      @JsonProperty("attributes") final Map<String, Object> attributes)
  {
    this.id = id;
    this.domain = domain;
    this.type = type;
    this.context = context;
    this.timestamp = timestamp;
    this.initiator = initiator;
    this.nodeId = nodeId;
    this.attributes = attributes;
  }

  public Long getId() {
    return id;
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

  public OffsetDateTime getTimestamp() {
    return timestamp;
  }

  public String getInitiator() {
    return initiator;
  }

  public String getNodeId() {
    return nodeId;
  }

  public Map<String, Object> getAttributes() {
    return attributes;
  }
}
