
package org.globalart.astrum.audit.internal;

import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jdk8.Jdk8Module;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;

/**
 * Simple DTO for writing audit data to log file in JSON format
 *
 * @since 3.16
 */
@JsonInclude(Include.NON_NULL)
public class AuditDTO
{
  private String timestamp;

  private String nodeId;

  private String initiator;

  private String domain;

  private String type;

  private String context;

  private String thread;

  private Map<String, Object> attributes;

  private static final DateTimeFormatter DATE_FORMAT = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss,SSSZ");

  private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper()
      .registerModule(new Jdk8Module())
      .registerModule(new JavaTimeModule());

  public AuditDTO() {
    // deserialization
  }

  public AuditDTO(final AuditData auditData) {
    if (auditData.getTimestamp() != null) {
      this.timestamp =
          auditData.getTimestamp().toInstant().atZone(ZoneId.systemDefault()).toOffsetDateTime().format(DATE_FORMAT);
    }
    this.nodeId = auditData.getNodeId();
    this.initiator = auditData.getInitiator();
    this.domain = auditData.getDomain();
    this.type = auditData.getType();
    this.context = auditData.getContext();
    this.thread = Thread.currentThread().getName();
    this.attributes = auditData.getAttributes();
  }

  public String getTimestamp() {
    return timestamp;
  }

  public void setTimestamp(String timestamp) {
    this.timestamp = timestamp;
  }

  public String getNodeId() {
    return nodeId;
  }

  public void setNodeId(String nodeId) {
    this.nodeId = nodeId;
  }

  public String getInitiator() {
    return initiator;
  }

  public void setInitiator(String initiator) {
    this.initiator = initiator;
  }

  public String getDomain() {
    return domain;
  }

  public void setDomain(String domain) {
    this.domain = domain;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  public String getContext() {
    return context;
  }

  public void setContext(String context) {
    this.context = context;
  }

  public String getThread() {
    return thread;
  }

  public Map<String, Object> getAttributes() {
    return attributes;
  }

  public void setAttributes(Map<String, Object> attributes) {
    this.attributes = attributes;
  }

  @Override
  public String toString() {
    return OBJECT_MAPPER.valueToTree(this).toString();
  }
}
