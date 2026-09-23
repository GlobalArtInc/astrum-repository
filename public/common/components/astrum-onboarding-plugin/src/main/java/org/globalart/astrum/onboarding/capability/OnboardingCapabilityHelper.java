
package org.globalart.astrum.onboarding.capability;

import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class OnboardingCapabilityHelper
{
  private final CapabilityRegistry capabilityRegistry;

  @Autowired
  public OnboardingCapabilityHelper(final CapabilityRegistry capabilityRegistry) {
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
  }

  public OnboardingCapability getOnboardingCapability() {
    Optional<? extends CapabilityReference> optionalCapabilityReference = capabilityRegistry.getAll()
        .stream()
        .filter(reference -> reference.context().type().toString().equals(OnboardingCapability.TYPE_ID))
        .findFirst();
    return optionalCapabilityReference.map(reference -> reference.capabilityAs(OnboardingCapability.class))
        .orElseThrow(() -> new IllegalStateException("OnboardingCapability not found"));
  }
}
