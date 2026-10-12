package org.globalart.astrum.repository.rubygems.internal.security;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.rubygems.internal.RubygemsResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.security.anonymous.AnonymousHelper;
import org.globalart.astrum.security.authc.AstrumBasicHttpAuthenticationFilter;
import org.globalart.astrum.security.realm.RealmManager;

import jakarta.inject.Provider;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
public class RubygemsApiKeyHandler
    implements Handler
{
  private final RubygemsApiKeyManager keyManager;

  private final Provider<RealmManager> realmManager;

  @Autowired
  public RubygemsApiKeyHandler(final RubygemsApiKeyManager keyManager, final Provider<RealmManager> realmManager) {
    this.keyManager = checkNotNull(keyManager);
    this.realmManager = checkNotNull(realmManager);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    if (isAnonymous()) {
      return HttpResponses.unauthorized("Authentication required to obtain a RubyGems API key", "BASIC",
          AstrumBasicHttpAuthenticationFilter.BASIC_AUTH_REALM);
    }
    if (HttpMethods.DELETE.equals(context.getRequest().getAction())) {
      return keyManager.deleteToken()
          ? RubygemsResponses.text(200, "RubyGems API key revoked")
          : RubygemsResponses.text(404, "No RubyGems API key to revoke");
    }
    if (!realmManager.get().isRealmEnabled(RubygemsApiKey.NAME)) {
      return RubygemsResponses.text(409, "The RubyGems API Key Realm is not active; ask an administrator to "
          + "activate it, or use 'Basic <base64 of user:password>' as the API key");
    }
    return HttpResponses.ok(new StringPayload(keyManager.getKeyForCurrentUser(), TEXT_PLAIN));
  }

  static boolean isAnonymous() {
    Subject subject = SecurityUtils.getSubject();
    return subject == null || subject.getPrincipal() == null || AnonymousHelper.isAnonymous(subject);
  }
}
