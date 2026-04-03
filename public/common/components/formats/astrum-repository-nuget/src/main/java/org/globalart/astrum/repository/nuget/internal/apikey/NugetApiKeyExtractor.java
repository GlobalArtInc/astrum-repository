package org.globalart.astrum.repository.nuget.internal.apikey;

import javax.annotation.Nullable;

import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.authc.apikey.ApiKeyExtractor;
import org.globalart.astrum.security.realm.RealmManager;

import com.google.common.net.HttpHeaders;
import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(NugetApiKeyRealm.NAME)
public class NugetApiKeyExtractor
    implements ApiKeyExtractor
{
  private final Provider<RealmManager> realmManager;

  @Autowired
  public NugetApiKeyExtractor(final Provider<RealmManager> realmManager) {
    this.realmManager = checkNotNull(realmManager);
  }

  @Nullable
  @Override
  public String extract(final HttpServletRequest request) {
    String apiKey = request.getHeader(NugetApiKeyRealm.HEADER);
    if (apiKey == null || apiKey.isBlank()) {
      return null;
    }
    String authorization = request.getHeader(HttpHeaders.AUTHORIZATION);
    if (authorization != null && !authorization.isBlank()) {
      return null;
    }
    if (!realmManager.get().isRealmEnabled(NugetApiKeyRealm.NAME)) {
      return null;
    }
    return apiKey.trim();
  }
}
