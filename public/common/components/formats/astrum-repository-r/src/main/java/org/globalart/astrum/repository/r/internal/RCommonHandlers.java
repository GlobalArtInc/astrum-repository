
package org.globalart.astrum.repository.r.internal;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Handler;

import org.springframework.stereotype.Component;

@Component
public class RCommonHandlers
{
  public final Handler notSupportedMetadataRequest =
      context -> HttpResponses.notFound("This metadata type is not supported for now.");
}
