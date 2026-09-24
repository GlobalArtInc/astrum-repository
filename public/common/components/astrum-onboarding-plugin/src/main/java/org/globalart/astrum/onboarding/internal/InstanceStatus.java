
package org.globalart.astrum.onboarding.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.onboarding.capability.OnboardingCapability;
import org.globalart.astrum.onboarding.capability.OnboardingCapabilityHelper;
import org.globalart.astrum.security.anonymous.AnonymousManager;

import static java.util.Objects.requireNonNull;
import org.springframework.stereotype.Component;

@Component
public class InstanceStatus
{
  private final AnonymousManager anonymousManager;

  private final OnboardingCapabilityHelper onboardingCapabilityHelper;

  @Autowired
  public InstanceStatus(
      final AnonymousManager anonymousManager,
      final OnboardingCapabilityHelper onboardingCapabilityHelper)
  {
    this.anonymousManager = requireNonNull(anonymousManager);
    this.onboardingCapabilityHelper = requireNonNull(onboardingCapabilityHelper);
  }

  public boolean isNew() {
    if (!anonymousManager.isConfigured()) {
      return true;
    }
    OnboardingCapability onboardingCapability = onboardingCapabilityHelper.getOnboardingCapability();
    return onboardingCapability.isRegistrationStarted() && !onboardingCapability.isRegistrationCompleted();
  }

  public boolean isUpgraded() {
    OnboardingCapability onboardingCapability = onboardingCapabilityHelper.getOnboardingCapability();
    return anonymousManager.isConfigured() && !onboardingCapability.isRegistrationStarted();
  }
}
