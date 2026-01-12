
package org.globalart.astrum.capability;

import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Delete;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Type of a capability.
 *
 * @since capabilities 2.0
 */
public class CapabilityType
{

  @NotNull
  @CapabilityTypeExists
  @CapabilityTypeIsExposed(groups = {Create.class, Delete.class, Default.class})
  private final String typeId;

  public CapabilityType(final String typeId) {
    this.typeId = checkNotNull(typeId);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof CapabilityType)) {
      return false;
    }

    final CapabilityType that = (CapabilityType) o;

    if (typeId != null ? !typeId.equals(that.typeId) : that.typeId != null) {
      return false;
    }

    return true;
  }

  @Override
  public int hashCode() {
    return typeId != null ? typeId.hashCode() : 0;
  }

  @Override
  public String toString() {
    return typeId;
  }

  public static CapabilityType capabilityType(final String typeId) {
    return new CapabilityType(typeId);
  }

}
