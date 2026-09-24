
package org.globalart.astrum.onboarding.internal;

import org.springframework.beans.factory.annotation.Autowired;
import java.util.Map;
import java.util.Optional;
import org.globalart.astrum.onboarding.OnboardingItem;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;

import static com.google.common.base.Preconditions.checkNotNull;

public abstract class CommunityOnboardingItem
    implements OnboardingItem
{
  private final String COMMUNITY = "COMMUNITY";

  private final String EULA_KEY = "astrum.community.eula.accepted";

  protected final ApplicationVersion applicationVersion;

  protected final GlobalKeyValueStore globalKeyValueStore;

  @Autowired
  public CommunityOnboardingItem(
      final ApplicationVersion applicationVersion,
      final GlobalKeyValueStore globalKeyValueStore)
  {
    this.applicationVersion = checkNotNull(applicationVersion);
    this.globalKeyValueStore = checkNotNull(globalKeyValueStore);
  }

  @Override
  public boolean applies() {
    boolean isCommunity = applicationVersion.getEdition().equalsIgnoreCase(COMMUNITY);
    boolean accepted = false;

    Optional<AstrumKeyValue> eulaStatusOptional = globalKeyValueStore.getKey(EULA_KEY);
    if (eulaStatusOptional.isPresent()) {
      AstrumKeyValue eulaStatus = eulaStatusOptional.get();
      Map<String, Object> eulaObject = eulaStatus.value();
      accepted = (Boolean) eulaObject.get("accepted");
    }

    return isCommunity && !accepted;
  }
}
