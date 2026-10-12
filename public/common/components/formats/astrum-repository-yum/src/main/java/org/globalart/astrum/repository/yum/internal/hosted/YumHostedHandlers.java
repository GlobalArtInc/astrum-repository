package org.globalart.astrum.repository.yum.internal.hosted;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.yum.internal.YumPaths;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.internal.YumPaths.requestPath;

@Component
public class YumHostedHandlers
{
  public final Handler getRepomd = context -> context.getRepository()
      .facet(YumHostedFacet.class)
      .getRepomd(YumPaths.baseOfMetadata(requestPath(context)).orElse(""))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler getMetadata = context -> {
    String path = requestPath(context);
    YumHostedFacet hosted = context.getRepository().facet(YumHostedFacet.class);
    if (YumPaths.PUBLIC_KEY.equals(YumPaths.fileName(path))) {
      return hosted.getPublicKey().map(HttpResponses::ok).orElseGet(HttpResponses::notFound);
    }
    return hosted.get(path).map(HttpResponses::ok).orElseGet(HttpResponses::notFound);
  };

  public final Handler get = context -> context.getRepository()
      .facet(YumHostedFacet.class)
      .get(requestPath(context))
      .map(HttpResponses::ok)
      .orElseGet(HttpResponses::notFound);

  public final Handler put = context -> {
    context.getRepository()
        .facet(YumHostedFacet.class)
        .put(requestPath(context), context.getRequest().getPayload());
    return HttpResponses.created();
  };

  public final Handler delete = context -> context.getRepository()
      .facet(YumHostedFacet.class)
      .delete(requestPath(context)) ? HttpResponses.noContent() : HttpResponses.notFound();
}
