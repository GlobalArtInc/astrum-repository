
package org.globalart.astrum.rapture.internal.state;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.security.realm.RealmManager;

import com.google.common.collect.ImmutableMap;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.security.internal.DefaultRealmConstants.DEFAULT_REALM_NAME;

/**
 * Contributes Local Authentication realm state information.
 *
 */
@Component
public class LocalAuthStateContributor
    implements StateContributor
{
  private final RealmManager realmManager;

  @Autowired
  public LocalAuthStateContributor(final RealmManager realmManager) {
    this.realmManager = realmManager;
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    // DEFAULT_REALM_NAME refers to "AstrumAuthenticatingRealm" - the local authentication realm
    return ImmutableMap.of("localAuthRealmEnabled", realmManager.isRealmEnabled(DEFAULT_REALM_NAME));
  }
}
