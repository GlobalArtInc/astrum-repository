package org.globalart.astrum.repository.terraform.internal.security;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.security.authc.AstrumBasicHttpAuthenticationFilter;
import org.globalart.astrum.security.realm.RealmManager;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Hands the signed-in user the bearer token to put into the Terraform CLI configuration, and revokes it on
 * {@code DELETE}. The token works for every Terraform repository the user may read.
 */
@Component
public class TerraformTokenHandler
    implements Handler
{
  private final TerraformTokenManager tokenManager;

  private final Provider<RealmManager> realmManager;

  @Autowired
  public TerraformTokenHandler(final TerraformTokenManager tokenManager, final Provider<RealmManager> realmManager) {
    this.tokenManager = checkNotNull(tokenManager);
    this.realmManager = checkNotNull(realmManager);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    if (TerraformRepositoryConfig.isAnonymous()) {
      return HttpResponses.unauthorized("Authentication required to obtain a Terraform token", "BASIC",
          AstrumBasicHttpAuthenticationFilter.BASIC_AUTH_REALM);
    }
    if (HttpMethods.DELETE.equals(context.getRequest().getAction())) {
      return tokenManager.deleteToken()
          ? HttpResponses.noContent()
          : TerraformResponses.error(404, "There is no Terraform token to revoke");
    }
    if (!realmManager.get().isRealmEnabled(TerraformToken.NAME)) {
      return TerraformResponses.error(409, "The Terraform Bearer Token Realm is not active; ask an administrator "
          + "to activate it in Security > Realms");
    }
    var body = TerraformResponses.object();
    body.put("token", tokenManager.getTokenForCurrentUser());
    return TerraformResponses.json(body);
  }
}
