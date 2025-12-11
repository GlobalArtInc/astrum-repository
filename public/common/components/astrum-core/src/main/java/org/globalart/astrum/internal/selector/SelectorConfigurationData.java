
package org.globalart.astrum.internal.selector;

import java.util.Map;
import java.util.Map.Entry;
import java.util.Objects;

import org.globalart.astrum.common.entity.HasName;
import org.globalart.astrum.selector.Selector;
import org.globalart.astrum.selector.SelectorConfiguration;

import static java.util.stream.Collectors.toMap;

/**
 * {@link Selector} configuration data.
 *
 * @since 3.21
 */
public class SelectorConfigurationData
    implements HasName, SelectorConfiguration
{
  private String name;

  private String type;

  private String description;

  private Map<String, String> attributes;

  @Override
  public String getName() {
    return name;
  }

  @Override
  public void setName(final String name) {
    this.name = name;
  }

  @Override
  public String getType() {
    return type;
  }

  @Override
  public void setType(final String type) {
    this.type = type;
  }

  @Override
  public String getDescription() {
    return description;
  }

  @Override
  public void setDescription(final String description) {
    this.description = description;
  }

  @Override
  public Map<String, String> getAttributes() {
    return attributes;
  }

  @Override
  public void setAttributes(final Map<String, ?> attributes) {
    this.attributes = attributes.entrySet().stream().collect(toMap(Entry::getKey, e -> e.getValue().toString()));
  }

  @Override
  public boolean equals(final Object other) {
    if (other instanceof SelectorConfigurationData) {
      SelectorConfigurationData o = (SelectorConfigurationData) other;
      return Objects.equals(name, o.name)
          && Objects.equals(type, o.type)
          && Objects.equals(description, o.description)
          && Objects.equals(attributes, o.attributes);
    }
    return false;
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, type, description, attributes);
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "name='" + name + '\'' +
        ", type='" + type + '\'' +
        ", description='" + description + '\'' +
        ", attributes='" + attributes + '\'' +
        '}';
  }
}
