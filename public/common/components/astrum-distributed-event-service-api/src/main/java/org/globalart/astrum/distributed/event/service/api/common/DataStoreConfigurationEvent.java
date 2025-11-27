
package org.globalart.astrum.distributed.event.service.api.common;

import java.util.Map;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.distributed.event.service.api.EventType.UPDATED;

/**
 * Indicates that a Data store configuration has been updated.
 *
 * @since 3.38
 */
public class DataStoreConfigurationEvent
    extends DistributedEventSupport
{
  public static final String NAME = "DataStoreConfigurationEvent";

  private final String configurationName;

  private final String type;

  private final String source;

  private final Map<String, String> attributes;

  @JsonCreator
  public DataStoreConfigurationEvent(
      @JsonProperty("configurationName") final String configurationName,
      @JsonProperty("type") final String type,
      @JsonProperty("source") final String source,
      @JsonProperty("attributes") final Map<String, String> attributes)
  {
    super(UPDATED);
    this.configurationName = checkNotNull(configurationName);
    this.type = checkNotNull(type);
    this.source = checkNotNull(source);
    this.attributes = checkNotNull(attributes);
  }

  public String getConfigurationName() {
    return configurationName;
  }

  public String getType() {
    return type;
  }

  public String getSource() {
    return source;
  }

  public Map<String, String> getAttributes() {
    return attributes;
  }

  @Override
  public String toString() {
    return "DataStoreConfigurationEvent{" +
        "name='" + configurationName + '\'' +
        ", type='" + type + '\'' +
        ", source='" + source + '\'' +
        ", attributes=" + attributes +
        '}';
  }
}
