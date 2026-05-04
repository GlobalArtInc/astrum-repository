/*
 * Copyright 2019, Imperva, Inc. All rights reserved.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License Version 1.0, which accompanies this
 * distribution and is available at http://www.eclipse.org/legal/epl-v10.html.
 *
 * Imperva, the Imperva logo, SecureSphere, Incapsula, CounterBreach,
 * ThreatRadar, Camouflage, Attack Analytics, Prevoty and design are trademarks
 * of Imperva, Inc. and its subsidiaries. All other brand or product names are
 * trademarks or registered trademarks of their respective owners.
 */
package org.globalart.astrum.repository.cargo.internal.security;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cargo.internal.CargoRepositoryConfig;
import org.globalart.astrum.repository.cargo.internal.CargoResponses;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.security.authc.AstrumBasicHttpAuthenticationFilter;
import org.globalart.astrum.security.realm.RealmManager;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
public class CargoTokenHandler
    implements Handler
{
  private final CargoTokenManager tokenManager;

  private final Provider<RealmManager> realmManager;

  @Autowired
  public CargoTokenHandler(final CargoTokenManager tokenManager, final Provider<RealmManager> realmManager) {
    this.tokenManager = checkNotNull(tokenManager);
    this.realmManager = checkNotNull(realmManager);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    if (CargoRepositoryConfig.isAnonymous()) {
      return HttpResponses.unauthorized("Authentication required to obtain a Cargo token", "BASIC",
          AstrumBasicHttpAuthenticationFilter.BASIC_AUTH_REALM);
    }
    if (HttpMethods.DELETE.equals(context.getRequest().getAction())) {
      return tokenManager.deleteToken() ? CargoResponses.ok() : CargoResponses.error(404, "no Cargo token to revoke");
    }
    if (!realmManager.get().isRealmEnabled(CargoToken.NAME)) {
      return CargoResponses.error(409, "the Cargo Bearer Token Realm is not active; ask an administrator to activate "
          + "it, or use 'Basic <base64 of user:password>' as the Cargo token");
    }
    return HttpResponses.ok(new StringPayload(tokenManager.getTokenForCurrentUser() + "\n", TEXT_PLAIN));
  }
}
