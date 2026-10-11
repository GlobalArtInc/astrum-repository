package org.globalart.astrum.accesstoken.internal;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.security.realm.RealmManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;

/**
 * Puts the access token realm first among the active realms once, the first time this version starts.
 *
 * Being first means a valid token is never offered as a password to a directory. An administrator may disable or move
 * the realm afterwards; that choice is kept.
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class AccessTokenRealmActivator
    extends StateGuardLifecycleSupport
{
  static final String ACTIVATED_KEY = "astrum.accesstoken.realm.activated";

  private final RealmManager realmManager;

  private final GlobalKeyValueStore keyValueStore;

  @Autowired
  public AccessTokenRealmActivator(final RealmManager realmManager, final GlobalKeyValueStore keyValueStore) {
    this.realmManager = checkNotNull(realmManager);
    this.keyValueStore = checkNotNull(keyValueStore);
  }

  @Override
  protected void doStart() {
    if (keyValueStore.getBoolean(ACTIVATED_KEY).orElse(false)) {
      return;
    }
    if (!realmManager.isRealmEnabled(AccessTokenRealm.NAME)) {
      realmManager.enableRealm(AccessTokenRealm.NAME, 0);
      log.info("Enabled the {}", AccessTokenRealm.NAME);
    }
    keyValueStore.setBoolean(ACTIVATED_KEY, true);
  }
}
