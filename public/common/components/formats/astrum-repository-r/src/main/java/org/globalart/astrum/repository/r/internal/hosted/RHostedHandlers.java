
package org.globalart.astrum.repository.r.internal.hosted;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Handler;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.r.internal.util.RPathUtils.requestPath;

@Component
public class RHostedHandlers
{
  public final Handler getPackages = context -> context.getRepository()
      .facet(RHostedFacet.class)
      .getPackages(requestPath(context))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler getArchive = context -> context.getRepository()
      .facet(RHostedFacet.class)
      .getArchive(requestPath(context))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler putArchive = context -> {
    context.getRepository()
        .facet(RHostedFacet.class)
        .upload(requestPath(context), context.getRequest().getPayload(), path -> { });
    return HttpResponses.ok();
  };

  public final Handler deleteArchive = context -> context.getRepository()
      .facet(RHostedFacet.class)
      .delete(requestPath(context)) ? HttpResponses.noContent() : HttpResponses.notFound();
}
