
package org.globalart.astrum.repository.conan.internal.common;

import java.util.function.Function;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.conan.internal.ConanJson;
import org.globalart.astrum.repository.conan.internal.security.token.ConanToken;
import org.globalart.astrum.repository.conan.internal.security.token.ConanTokenManager;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.security.realm.RealmManager;

import jakarta.inject.Provider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpStatus.FORBIDDEN;
import static org.globalart.astrum.repository.http.HttpStatus.OK;
import static org.globalart.astrum.repository.http.HttpStatus.UNAUTHORIZED;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
public class ConanUserHandlers
{
  public static final String CAPABILITIES_HEADER = "X-Conan-Server-Capabilities";

  public static final String REVISIONS = "revisions";

  private static final String REALM_INACTIVE =
      "The Conan Bearer Token Realm is not active; enable it under Security > Realms";

  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ConanTokenManager tokenManager;

  private final Provider<RealmManager> realmManager;

  public final Handler authenticate = this::authenticate;

  public final Handler checkCredentials = this::checkCredentials;

  @Autowired
  public ConanUserHandlers(final ConanTokenManager tokenManager, final Provider<RealmManager> realmManager) {
    this.tokenManager = checkNotNull(tokenManager);
    this.realmManager = checkNotNull(realmManager);
  }

  public Handler ping(final Function<Repository, String> capabilities) {
    return context -> new Response.Builder()
        .status(Status.success(OK))
        .header(CAPABILITIES_HEADER, capabilities.apply(context.getRepository()))
        .build();
  }

  private Response authenticate(final Context context) {
    if (!realmEnabled()) {
      return text(FORBIDDEN, REALM_INACTIVE);
    }
    String token = tokenManager.login();
    if (token == null) {
      return text(UNAUTHORIZED, "Bad username or password");
    }
    log.debug("Issued a Conan token in repository {}", context.getRepository().getName());
    return text(OK, token);
  }

  private Response checkCredentials(final Context context) {
    String user = tokenManager.user();
    if (user != null) {
      return text(OK, user);
    }
    String authorization = ConanJson.header(context, "Authorization");
    if (authorization != null && authorization.startsWith("Bearer " + ConanToken.NAME + ".") && !realmEnabled()) {
      return text(UNAUTHORIZED, REALM_INACTIVE);
    }
    return text(UNAUTHORIZED, "Unknown user");
  }

  private boolean realmEnabled() {
    return realmManager.get().isRealmEnabled(ConanToken.NAME);
  }

  private static Response text(final int status, final String message) {
    return new Response.Builder()
        .status(status == OK ? Status.success(status) : Status.failure(status, message))
        .payload(new StringPayload(message, TEXT_PLAIN))
        .build();
  }
}
