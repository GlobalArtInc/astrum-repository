
package org.globalart.astrum.onboarding.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.onboarding.OnboardingItemPriority;
import org.springframework.stereotype.Component;

@Component
public class CommunityEulaOnboardingItem
    extends CommunityOnboardingItem
{
  @Autowired
  public CommunityEulaOnboardingItem(
      final ApplicationVersion applicationVersion,
      final GlobalKeyValueStore globalKeyValueStore)
  {
    super(applicationVersion, globalKeyValueStore);
  }

  @Override
  public String getType() {
    return "CommunityEula";
  }

  @Override
  public int getPriority() {
    return OnboardingItemPriority.COMMUNITY_EULA_ONBOARDING;
  }
}
