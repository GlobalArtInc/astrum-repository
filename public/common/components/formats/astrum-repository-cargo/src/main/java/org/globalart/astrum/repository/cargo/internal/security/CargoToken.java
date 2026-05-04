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

import javax.annotation.Nullable;

import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.token.BearerToken;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.net.HttpHeaders.AUTHORIZATION;

@Component
@Qualifier(CargoToken.NAME)
public class CargoToken
    extends BearerToken
{
  public static final String NAME = "CargoToken";

  private static final String PREFIX = NAME + ".";

  public CargoToken() {
    super(NAME);
  }

  @Nullable
  @Override
  public String extract(final HttpServletRequest request) {
    String bearer = super.extract(request);
    if (bearer != null) {
      return bearer;
    }
    String header = request.getHeader(AUTHORIZATION);
    if (header == null) {
      return null;
    }
    String value = header.strip();
    if (value.startsWith(PREFIX) && value.length() > PREFIX.length() && value.indexOf(' ') < 0) {
      return value.substring(PREFIX.length());
    }
    return null;
  }
}
