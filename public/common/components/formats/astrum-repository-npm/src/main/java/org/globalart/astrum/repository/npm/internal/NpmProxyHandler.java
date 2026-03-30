
package org.globalart.astrum.repository.npm.internal;

import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import org.springframework.stereotype.Component;

/**
 * Npm specific handling for proxy repositories.
 * 
 * @since 3.0
 */
@Component
public class NpmProxyHandler
    extends ProxyHandler
{
  @Override
  protected Response buildNotFoundResponse(final Context context) {
    State state = context.getAttributes().require(TokenMatcher.State.class);
    NpmPackageId packageId = NpmPaths.packageId(state);
    return NpmResponses.packageNotFound(packageId);
  }
}
