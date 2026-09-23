
package org.globalart.astrum.onboarding.capability;

import java.util.Map;

import org.globalart.astrum.capability.CapabilityConfigurationSupport;

import com.google.common.collect.Maps;

import static com.google.common.base.Preconditions.checkNotNull;

public class OnboardingCapabilityConfiguration
    extends CapabilityConfigurationSupport
{
  public static final boolean DEFAULT_REGISTRATION_STARTED = false;

  public static final String REGISTRATION_STARTED = "registrationStarted";

  public static final boolean DEFAULT_REGISTRATION_COMPLETED = false;

  public static final String REGISTRATION_COMPLETED = "registrationCompleted";

  private boolean registrationStarted;

  private boolean registrationCompleted;

  public OnboardingCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    this.registrationStarted = parseBoolean(properties.get(REGISTRATION_STARTED), DEFAULT_REGISTRATION_STARTED);
    this.registrationCompleted = parseBoolean(properties.get(REGISTRATION_COMPLETED), DEFAULT_REGISTRATION_COMPLETED);
  }

  public boolean isRegistrationStarted() {
    return registrationStarted;
  }

  public OnboardingCapabilityConfiguration setRegistrationStarted(final boolean registrationStarted) {
    this.registrationStarted = registrationStarted;
    return this;
  }

  public boolean isRegistrationCompleted() {
    return registrationCompleted;
  }

  public OnboardingCapabilityConfiguration setRegistrationCompleted(final boolean registrationCompleted) {
    this.registrationCompleted = registrationCompleted;
    return this;
  }

  public Map<String, String> asMap() {
    final Map<String, String> properties = Maps.newHashMap();
    properties.put(REGISTRATION_STARTED, String.valueOf(registrationStarted));
    properties.put(REGISTRATION_COMPLETED, String.valueOf(registrationCompleted));
    return properties;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "registrationStarted=" + registrationStarted + "; " + "registrationCompleted=" + registrationCompleted + "; " +
        "}";
  }
}
