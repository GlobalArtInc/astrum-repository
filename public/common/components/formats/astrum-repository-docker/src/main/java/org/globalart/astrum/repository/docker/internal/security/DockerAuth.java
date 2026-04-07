package org.globalart.astrum.repository.docker.internal.security;

import java.net.URI;

import javax.annotation.Nullable;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.internal.v2.DockerError;
import org.globalart.astrum.repository.docker.internal.v2.DockerResponses;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.security.anonymous.AnonymousHelper;
import org.globalart.astrum.security.authc.AstrumBasicHttpAuthenticationFilter;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.token.BearerTokenRealm;

import jakarta.inject.Provider;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class DockerAuth
{
  public static final String PREFIX_ATTRIBUTE = "astrum.docker.v2.prefix";

  public static final String NAME_PREFIX_ATTRIBUTE = "astrum.docker.v2.namePrefix";

  public static final String REALM_ATTRIBUTE = "astrum.docker.v2.realm";

  public static final String SERVICE_ATTRIBUTE = "astrum.docker.v2.service";

  public static final String BASIC_CHALLENGE =
      "Basic realm=\"" + AstrumBasicHttpAuthenticationFilter.BASIC_AUTH_REALM + "\"";

  private static final Logger log = LoggerFactory.getLogger(DockerAuth.class);

  private final Provider<RealmManager> realmManager;

  @Autowired
  public DockerAuth(final Provider<RealmManager> realmManager) {
    this.realmManager = checkNotNull(realmManager);
  }

  public boolean tokenRealmEnabled() {
    try {
      return realmManager.get().isRealmEnabled(DockerTokenRealm.NAME);
    }
    catch (RuntimeException e) {
      log.debug("Unable to determine whether the docker token realm is enabled", e);
      return false;
    }
  }

  public static boolean isAnonymous() {
    Subject subject = SecurityUtils.getSubject();
    return subject == null || subject.getPrincipal() == null || AnonymousHelper.isAnonymous(subject);
  }

  public static boolean isTokenAuthenticated(final AttributesMap attributes) {
    return Boolean.TRUE.equals(attributes.get(BearerTokenRealm.IS_TOKEN_AUTH_KEY));
  }

  public String challenge(
      @Nullable final Repository repository,
      @Nullable final String realm,
      @Nullable final String service,
      @Nullable final String scope)
  {
    boolean bearer = tokenRealmEnabled() && (repository == null || !DockerConfig.forceBasicAuth(repository));
    if (!bearer) {
      return BASIC_CHALLENGE;
    }
    String realmUrl = realm != null ? realm : defaultRealm();
    String serviceName = service != null ? service : defaultService();
    StringBuilder header = new StringBuilder("Bearer realm=\"").append(realmUrl).append('"');
    if (serviceName != null) {
      header.append(",service=\"").append(serviceName).append('"');
    }
    if (scope != null) {
      header.append(",scope=\"").append(scope).append('"');
    }
    return header.toString();
  }

  public String challenge(
      @Nullable final Repository repository,
      final AttributesMap attributes,
      @Nullable final String scope)
  {
    return challenge(repository, string(attributes.get(REALM_ATTRIBUTE)), string(attributes.get(SERVICE_ATTRIBUTE)),
        scope);
  }

  public Response unauthorized(
      @Nullable final Repository repository,
      final AttributesMap attributes,
      @Nullable final String scope)
  {
    Response error = DockerResponses.error(DockerError.UNAUTHORIZED);
    return new Response.Builder()
        .copy(error)
        .header(DockerResponses.WWW_AUTHENTICATE, challenge(repository, attributes, scope))
        .build();
  }

  @Nullable
  private static String string(@Nullable final Object value) {
    return value == null ? null : value.toString();
  }

  private static String defaultRealm() {
    String base = BaseUrlHolder.isSet() ? BaseUrlHolder.get() : "";
    return base + "/v2/token";
  }

  @Nullable
  private static String defaultService() {
    if (!BaseUrlHolder.isSet()) {
      return null;
    }
    try {
      URI uri = URI.create(BaseUrlHolder.get());
      return uri.getPort() > 0 ? uri.getHost() + ":" + uri.getPort() : uri.getHost();
    }
    catch (IllegalArgumentException e) {
      return null;
    }
  }
}
