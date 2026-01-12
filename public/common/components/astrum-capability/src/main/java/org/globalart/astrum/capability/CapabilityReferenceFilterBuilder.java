
package org.globalart.astrum.capability;

import java.util.HashMap;
import java.util.Map;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.base.Predicate;

/**
 * Helper to build {@link CapabilityReferenceFilter}.
 */
public class CapabilityReferenceFilterBuilder
{
  private CapabilityReferenceFilterBuilder() {
    // empty
  }

  public static CapabilityReferenceFilter capabilities() {
    return new CapabilityReferenceFilter();
  }

  public static class CapabilityReferenceFilter
      implements Predicate<CapabilityReference>
  {

    private String typeId;

    private CapabilityIdentity ignoreCapabilityId;

    private Boolean enabled;

    private Boolean active;

    private Boolean includeNotExposed = false;

    private Map<String, String> properties = new HashMap<String, String>();

    public String getTypeId() {
      return typeId;
    }

    @VisibleForTesting
    CapabilityIdentity getIgnoreCapabilityId() {
      return ignoreCapabilityId;
    }

    public Boolean isEnabled() {
      return enabled;
    }

    public Boolean isActive() {
      return active;
    }

    public Boolean isIncludeNotExposed() {
      return includeNotExposed;
    }

    public Map<String, String> getProperties() {
      return properties;
    }

    public CapabilityReferenceFilter withType(final CapabilityType type) {
      typeId = type.toString();
      return this;
    }

    /**
     * @since 3.13
     */
    public CapabilityReferenceFilter ignore(final CapabilityIdentity id) {
      ignoreCapabilityId = id;
      return this;
    }

    public CapabilityReferenceFilter enabled(boolean enabled) {
      this.enabled = enabled;
      return this;
    }

    public CapabilityReferenceFilter enabled() {
      enabled = true;
      return this;
    }

    public CapabilityReferenceFilter disabled() {
      enabled = false;
      return this;
    }

    public CapabilityReferenceFilter active(boolean active) {
      this.active = active;
      return this;
    }

    public CapabilityReferenceFilter active() {
      active = true;
      return this;
    }

    public CapabilityReferenceFilter passive() {
      active = false;
      return this;
    }

    public CapabilityReferenceFilter withBoundedProperty(final String key) {
      properties.put(key, null);
      return this;
    }

    public CapabilityReferenceFilter withProperty(final String key, final String value) {
      properties.put(key, value);
      return this;
    }

    public CapabilityReferenceFilter onRepository(final String key, final String repositoryId) {
      return withProperty(key, repositoryId);
    }

    public CapabilityReferenceFilter includeNotExposed() {
      includeNotExposed = true;
      return this;
    }

    @Override
    public boolean apply(final CapabilityReference input) {
      if (input == null) {
        return false;
      }
      if (input.context().descriptor() != null
          && !input.context().descriptor().isExposed() && !includeNotExposed) {
        return false;
      }
      if (typeId != null && !typeId.equals(input.context().type().toString())) {
        return false;
      }
      if (ignoreCapabilityId != null && ignoreCapabilityId.equals(input.context().id())) {
        return false;
      }
      if (enabled != null && !enabled.equals(input.context().isEnabled())) {
        return false;
      }
      if (active != null && !active.equals(input.context().isActive())) {
        return false;
      }
      if (properties != null && !properties.isEmpty()) {
        final Map<String, String> inputPropertiesMap = input.context().properties();
        if (inputPropertiesMap == null || inputPropertiesMap.isEmpty()) {
          return false;
        }

        for (Map.Entry<String, String> entry : properties.entrySet()) {
          final String key = entry.getKey();
          if (key == null) {
            return false;
          }
          final String value = entry.getValue();
          if (value == null) {
            if (!inputPropertiesMap.containsKey(key)) {
              return false;
            }
          }
          else {
            final String inputValue = inputPropertiesMap.get(key);
            if (inputValue == null || !value.equals(inputValue)) {
              return false;
            }
          }
        }
      }
      return true;
    }

  }

}
