package org.globalart.astrum.oidc.internal;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.security.realm.RealmManager;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class OidcStateContributor
    implements StateContributor
{
  private final OidcConfigurationStore configurationStore;

  private final RealmManager realmManager;

  @Autowired
  public OidcStateContributor(final OidcConfigurationStore configurationStore, final RealmManager realmManager) {
    this.configurationStore = checkNotNull(configurationStore);
    this.realmManager = checkNotNull(realmManager);
  }

  @Override
  @Nullable
  public Map<String, Object> getState() {
    boolean enabled = configurationStore.isConfigured() && realmManager.isRealmEnabled(OidcRealm.NAME);
    return Map.of("oauth2Available", true, "oauth2Enabled", enabled);
  }
}
