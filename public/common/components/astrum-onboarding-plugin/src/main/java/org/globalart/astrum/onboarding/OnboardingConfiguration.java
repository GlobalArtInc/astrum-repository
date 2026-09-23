
package org.globalart.astrum.onboarding;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
public class OnboardingConfiguration
{
  private boolean enabled;

  @Autowired
  public OnboardingConfiguration(
      @Value("${astrum.onboarding.enabled:true}") final boolean enabled)
  {
    this.enabled = enabled;
  }

  public boolean isEnabled() {
    return enabled;
  }
}
