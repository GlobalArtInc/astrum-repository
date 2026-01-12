
package org.globalart.astrum.capability.firewall;

import java.util.Collection;
import java.util.Optional;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;

import static org.globalart.astrum.capability.CapabilityReferenceFilterBuilder.capabilities;

public class FirewallCapability
{
  private static final String CLM_CAPABILITY_ID = "clm";

  private FirewallCapability() {
    throw new IllegalStateException("Utility class");
  }

  public static Optional<CapabilityContext> findFirewallCapability(final CapabilityRegistry capabilities) {
    Collection<? extends CapabilityReference> references = capabilities
        .get(capabilities().withType(CapabilityType.capabilityType(CLM_CAPABILITY_ID)).includeNotExposed());

    if (references.isEmpty()) {
      return Optional.empty();
    }

    CapabilityContext context = references.iterator().next().context();
    return Optional.of(context);
  }

  public static boolean isFirewallSupportedFormat(String format) {
    for (FirewallSupportedFormat supportedFormat : FirewallSupportedFormat.values()) {
      if (supportedFormat.getValue().equals(format)) {
        return true;
      }
    }
    return false;
  }
}
