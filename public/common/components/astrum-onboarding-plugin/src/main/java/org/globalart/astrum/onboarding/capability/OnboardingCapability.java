
package org.globalart.astrum.onboarding.capability;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Condition;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.onboarding.capability.OnboardingCapabilityDescriptor.messages;

@Component(OnboardingCapability.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class OnboardingCapability
    extends CapabilitySupport<OnboardingCapabilityConfiguration>

{
  public static final String TYPE_ID = "onboarding-wizard";

  public static final CapabilityType TYPE = CapabilityType.capabilityType(TYPE_ID);

  @Override
  protected OnboardingCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new OnboardingCapabilityConfiguration(properties);
  }

  @Nullable
  @Override
  protected String renderDescription() {
    return context().isActive() ? messages.enabled() : messages.disabled();
  }

  @Override
  public Condition activationCondition() {
    return conditions().logical()
        .and(conditions().astrum().active(), conditions().capabilities().capabilityHasNoDuplicates(),
            conditions().capabilities().passivateCapabilityDuringUpdate());
  }

  public boolean isRegistrationStarted() {
    return getConfig().isRegistrationStarted();
  }

  public void setRegistrationStarted(final boolean registrationStarted) {
    getConfig().setRegistrationStarted(registrationStarted);
  }

  public boolean isRegistrationCompleted() {
    return getConfig().isRegistrationCompleted();
  }

  public void setRegistrationCompleted(final boolean registrationCompleted) {
    getConfig().setRegistrationCompleted(registrationCompleted);
  }
}
