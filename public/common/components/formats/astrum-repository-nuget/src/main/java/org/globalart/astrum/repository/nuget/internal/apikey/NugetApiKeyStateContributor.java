package org.globalart.astrum.repository.nuget.internal.apikey;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.security.realm.RealmManager;

import com.google.common.collect.ImmutableMap;
import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class NugetApiKeyStateContributor
    implements StateContributor
{
  private final Provider<RealmManager> realmManager;

  @Autowired
  public NugetApiKeyStateContributor(final Provider<RealmManager> realmManager) {
    this.realmManager = checkNotNull(realmManager);
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of("nugetApiKeyRealmEnabled", realmManager.get().isRealmEnabled(NugetApiKeyRealm.NAME));
  }
}
