
package org.globalart.astrum.repository.conan.internal.security.token;

import org.globalart.astrum.security.token.BearerToken;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(ConanToken.NAME)
public class ConanToken
    extends BearerToken
{
  public static final String NAME = "ConanToken";

  public ConanToken() {
    super(NAME);
  }
}
