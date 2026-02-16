
package org.globalart.astrum.repository.httpbridge.legacy;

import java.util.Collection;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.common.property.SystemPropertiesHelper;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.capabilities;
import org.springframework.stereotype.Component;

@Component
public class LegacyUrlEnabledHelper
{
  private final boolean supportLegacyContent = SystemPropertiesHelper
      .getBoolean("org.globalart.astrum.repository.httpbridge.internal.HttpBridgeModule.legacy", false);

  private final CapabilityRegistry capabilities;

  @Autowired
  public LegacyUrlEnabledHelper(final CapabilityRegistry capabilities) {
    this.capabilities = checkNotNull(capabilities);
  }

  public boolean isEnabled() {
    return supportLegacyContent || isLegacyUrlCapabilityActive();
  }

  private boolean isLegacyUrlCapabilityActive() {
    Collection<? extends CapabilityReference> references = capabilities
        .get(capabilities().withType(LegacyUrlCapabilityDescriptor.TYPE));

    if (references.isEmpty()) {
      return false;
    }

    return references.iterator().next().context().isActive();
  }
}
