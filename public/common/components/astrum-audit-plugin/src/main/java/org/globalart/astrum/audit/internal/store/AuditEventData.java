
package org.globalart.astrum.audit.internal.store;

import java.time.OffsetDateTime;
import java.util.Map;

/**
 * Data entity for persisted audit events.
 */
public class AuditEventData
{
  private long id;

  private String domain;

  private String type;

  private String context;

  private OffsetDateTime timestamp;

  private String initiator;

  private String nodeId;

  private Map<String, Object> attributes;

  public long getId() {
    return id;
  }

  public void setId(final long id) {
    this.id = id;
  }

  public String getDomain() {
    return domain;
  }

  public void setDomain(final String domain) {
    this.domain = domain;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public String getContext() {
    return context;
  }

  public void setContext(final String context) {
    this.context = context;
  }

  public OffsetDateTime getTimestamp() {
    return timestamp;
  }

  public void setTimestamp(final OffsetDateTime timestamp) {
    this.timestamp = timestamp;
  }

  public String getInitiator() {
    return initiator;
  }

  public void setInitiator(final String initiator) {
    this.initiator = initiator;
  }

  public String getNodeId() {
    return nodeId;
  }

  public void setNodeId(final String nodeId) {
    this.nodeId = nodeId;
  }

  public Map<String, Object> getAttributes() {
    return attributes;
  }

  public void setAttributes(final Map<String, Object> attributes) {
    this.attributes = attributes;
  }
}
