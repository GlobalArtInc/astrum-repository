package org.globalart.astrum.repository.yum.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.yum.internal.YumPaths;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.internal.YumPaths.requestPath;

@Component
public class YumGroupRepomdHandler
    extends GroupHandler
{
  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    String base = YumPaths.baseOfMetadata(requestPath(context)).orElse("");
    return context.getRepository().facet(YumGroupFacet.class)
        .repomd(context, base)
        .map(repomd -> HttpResponses.ok(repomd.content()))
        .orElseGet(() -> notFoundResponse(context));
  }
}
