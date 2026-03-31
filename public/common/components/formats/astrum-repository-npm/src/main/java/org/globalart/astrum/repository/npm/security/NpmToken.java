
package org.globalart.astrum.repository.npm.security;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import java.util.List;


import org.globalart.astrum.security.token.BearerToken;

/**
 * npm API-Key; used by npm CLI to authenticate.
 */
@Component
@Qualifier(NpmToken.NAME)
public class NpmToken
    extends BearerToken
{
  public static final String NAME = "NpmToken";

  public NpmToken() {
    super(NAME);
  }

  @Override
  protected boolean matchesFormat(final List<String> parts) {
    return super.matchesFormat(parts) || !parts.get(1).contains(".");
  }
}
