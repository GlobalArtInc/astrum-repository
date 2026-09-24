
package org.globalart.astrum.onboarding.internal;

import java.util.Collections;
import java.util.Comparator;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.onboarding.OnboardingConfiguration;
import org.globalart.astrum.onboarding.OnboardingItem;
import org.globalart.astrum.onboarding.OnboardingManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
public class OnboardingManagerImpl
    implements OnboardingManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final OnboardingConfiguration onboardingConfiguration;

  private final Set<OnboardingItem> onboardingItems;

  @Autowired
  public OnboardingManagerImpl(
      final Set<OnboardingItem> onboardingItems,
      final OnboardingConfiguration onboardingConfiguration)
  {
    this.onboardingItems = checkNotNull(onboardingItems);
    this.onboardingConfiguration = checkNotNull(onboardingConfiguration);
  }

  @Override
  public boolean needsOnboarding() {
    return !getOnboardingItems().isEmpty();
  }

  @Override
  public List<OnboardingItem> getOnboardingItems() {
    if (onboardingConfiguration.isEnabled()) {
      return onboardingItems.stream()
          .filter(OnboardingItem::applies)
          .sorted(Comparator.comparingInt(OnboardingItem::getPriority))
          .collect(Collectors.toList());
    }
    return Collections.emptyList();
  }
}
