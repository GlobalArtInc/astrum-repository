
package org.globalart.astrum.onboarding.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.onboarding.OnboardingItem;
import org.globalart.astrum.onboarding.OnboardingItemPriority;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
public class ConfigureAnonymousAccessItem
    implements OnboardingItem
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AnonymousManager anonymousManager;

  @Autowired
  public ConfigureAnonymousAccessItem(final AnonymousManager anonymousManager) {
    this.anonymousManager = checkNotNull(anonymousManager);
  }

  @Override
  public String getType() {
    return "ConfigureAnonymousAccess";
  }

  @Override
  public int getPriority() {
    return OnboardingItemPriority.CONFIGURE_ANONYMOUS_ACCESS;
  }

  @Override
  public boolean applies() {
    return !anonymousManager.isConfigured();
  }
}
