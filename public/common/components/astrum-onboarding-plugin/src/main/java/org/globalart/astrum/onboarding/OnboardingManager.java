
package org.globalart.astrum.onboarding;

import java.util.List;

/**
 * Manage the onboarding process, will maintain a list of {@link OnboardingItem}s that need to be acted upon
 *
 * @since 3.17
 */
public interface OnboardingManager
{
  /**
   * Check if there are any {@link OnboardingItem}s that need processing
   */
  boolean needsOnboarding();

  /**
   * Retrieve list of {@link OnboardingItem}s that need to be processed
   */
  List<OnboardingItem> getOnboardingItems();
}
