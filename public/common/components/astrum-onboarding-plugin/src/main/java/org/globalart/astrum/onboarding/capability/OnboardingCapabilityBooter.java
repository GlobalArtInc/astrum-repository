
package org.globalart.astrum.onboarding.capability;

import org.globalart.astrum.capability.CapabilityBooterSupport;
import org.globalart.astrum.capability.CapabilityRegistry;

import com.google.common.collect.ImmutableMap;

import static org.globalart.astrum.onboarding.capability.OnboardingCapabilityConfiguration.DEFAULT_REGISTRATION_COMPLETED;
import static org.globalart.astrum.onboarding.capability.OnboardingCapabilityConfiguration.DEFAULT_REGISTRATION_STARTED;
import static org.globalart.astrum.onboarding.capability.OnboardingCapabilityConfiguration.REGISTRATION_COMPLETED;
import static org.globalart.astrum.onboarding.capability.OnboardingCapabilityConfiguration.REGISTRATION_STARTED;
import org.springframework.stereotype.Component;

@Component
public class OnboardingCapabilityBooter
    extends CapabilityBooterSupport
{
  @Override
  protected void boot(final CapabilityRegistry registry) throws Exception {
    maybeAddCapability(
        registry,
        OnboardingCapability.TYPE,
        true,
        null,
        ImmutableMap.of(
            REGISTRATION_STARTED, String.valueOf(DEFAULT_REGISTRATION_STARTED),
            REGISTRATION_COMPLETED, String.valueOf(DEFAULT_REGISTRATION_COMPLETED)));
  }
}
