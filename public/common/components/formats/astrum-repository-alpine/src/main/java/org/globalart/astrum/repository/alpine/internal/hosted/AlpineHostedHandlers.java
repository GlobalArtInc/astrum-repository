package org.globalart.astrum.repository.alpine.internal.hosted;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Handler;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.directory;
import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.fileName;
import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.requestPath;

@Component
public class AlpineHostedHandlers
{
  public final Handler getIndex = context -> context.getRepository()
      .facet(AlpineHostedFacet.class)
      .getIndex(directory(requestPath(context)))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler getPackage = context -> context.getRepository()
      .facet(AlpineHostedFacet.class)
      .getPackage(requestPath(context))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler putPackage = context -> {
    String path = requestPath(context);
    context.getRepository()
        .facet(AlpineHostedFacet.class)
        .put(directory(path), fileName(path), context.getRequest().getPayload(), checkedPath -> { });
    return HttpResponses.created();
  };

  public final Handler deletePackage = context -> context.getRepository()
      .facet(AlpineHostedFacet.class)
      .delete(requestPath(context)) ? HttpResponses.noContent() : HttpResponses.notFound();
}
