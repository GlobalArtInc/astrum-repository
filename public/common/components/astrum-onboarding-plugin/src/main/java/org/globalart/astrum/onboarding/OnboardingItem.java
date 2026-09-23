
package org.globalart.astrum.onboarding;

/**
 * An item that needs to be configured in the onboarding process in the UI, before other actions can be taken.
 *
 * @see OnboardingManager
 * @since 3.17
 */
public interface OnboardingItem
{
  /**
   * @return A unique string that defines the type of this onboarding item
   */
  String getType();

  /**
   * @return true if the item needs to be resolved, false if already done
   */
  boolean applies();

  /**
   * @return priority of the item in relation to other items (i.e. order shown in the onboarding wizard),
   *         {@link Integer#MIN_VALUE} is highest priority
   */
  int getPriority();
}
