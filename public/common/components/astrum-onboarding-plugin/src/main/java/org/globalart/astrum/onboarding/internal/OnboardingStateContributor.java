
package org.globalart.astrum.onboarding.internal;

import java.util.HashMap;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.globalart.astrum.onboarding.OnboardingConfiguration;
import org.globalart.astrum.onboarding.OnboardingManager;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.security.config.AdminPasswordFileManager;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_ONBOARDING_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_ONBOARDING_ENABLED_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class OnboardingStateContributor
    implements StateContributor
{
  private final OnboardingConfiguration onboardingConfiguration;

  private final OnboardingManager onboardingManager;

  private final AdminPasswordFileManager adminPasswordFileManager;

  private final boolean isReactOnboardingEnabled;

  private boolean needsOnboarding = true;

  @Autowired
  public OnboardingStateContributor(
      final OnboardingConfiguration onboardingConfiguration,
      final OnboardingManager onboardingManager,
      final AdminPasswordFileManager adminPasswordFileManager,
      @Value(REACT_ONBOARDING_ENABLED_NAMED_VALUE) final boolean isReactOnboardingEnabled)
  {
    this.onboardingConfiguration = checkNotNull(onboardingConfiguration);
    this.onboardingManager = checkNotNull(onboardingManager);
    this.adminPasswordFileManager = checkNotNull(adminPasswordFileManager);
    this.isReactOnboardingEnabled = isReactOnboardingEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    // cache the onboarding flag, once it's false there is no longer a need to check anymore
    needsOnboarding = needsOnboarding && onboardingConfiguration.isEnabled() && onboardingManager.needsOnboarding();

    Map<String, Object> properties = new HashMap<>();

    // Always publish the React onboarding flag so both ExtJS and React can read it via AS.State.
    properties.put(REACT_ONBOARDING_ENABLED, isReactOnboardingEnabled);

    if (needsOnboarding) {
      properties.put("onboarding.required", true);
    }

    if (adminPasswordFileManager.exists()) {
      properties.put("admin.password.file", adminPasswordFileManager.getPath());
    }

    return properties;
  }
}
