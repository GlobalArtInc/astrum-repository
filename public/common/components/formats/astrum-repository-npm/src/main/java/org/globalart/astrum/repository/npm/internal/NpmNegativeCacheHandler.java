
package org.globalart.astrum.repository.npm.internal;

import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import org.apache.http.HttpStatus;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Npm specific handling for negative cache responses.
 *
 * @since 3.0
 */
@Component
public class NpmNegativeCacheHandler
    extends NegativeCacheHandler
{
  @Autowired
  public NpmNegativeCacheHandler(final NodeAccess nodeAccess) {
    super(nodeAccess);
  }

  @Override
  protected Response buildResponse(final Status status, final Context context) {
    if (status.getCode() == HttpStatus.SC_NOT_FOUND) {
      State state = context.getAttributes().require(TokenMatcher.State.class);
      NpmPackageId packageId = NpmPaths.packageId(state);
      return NpmResponses.packageNotFound(packageId);
    }
    return super.buildResponse(status, context);
  }
}
