
package org.globalart.astrum.capability;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Identity of a capability.
 *
 * @since capabilities 2.0
 */
public class CapabilityIdentity
{

  private final String value;

  @JsonCreator
  public CapabilityIdentity(@JsonProperty("value") final String value) {
    this.value = checkNotNull(value);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof CapabilityIdentity)) {
      return false;
    }

    final CapabilityIdentity that = (CapabilityIdentity) o;

    if (value != null ? !value.equals(that.value) : that.value != null) {
      return false;
    }

    return true;
  }

  @Override
  public int hashCode() {
    return value != null ? value.hashCode() : 0;
  }

  @JsonProperty("value")
  @Override
  public String toString() {
    return value;
  }

  public static CapabilityIdentity capabilityIdentity(final String value) {
    return new CapabilityIdentity(value);
  }

}
