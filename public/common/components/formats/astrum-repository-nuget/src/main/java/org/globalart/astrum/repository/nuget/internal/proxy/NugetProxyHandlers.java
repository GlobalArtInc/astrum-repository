package org.globalart.astrum.repository.nuget.internal.proxy;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV3Documents;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.http.HttpResponses.notFound;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.json;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.repositoryUrl;

@Component
public class NugetProxyHandlers
{
  private static NugetProxyFacet facet(final Context context) {
    return context.getRepository().facet(NugetProxyFacet.class);
  }

  public final Handler serviceIndex = context -> facet(context).isV2()
      ? notFound()
      : json(NugetV3Documents.serviceIndex(repositoryUrl(context), false, false));

  public final Handler requireV3 = context -> facet(context).isV2() ? notFound() : context.proceed();

  public final Handler requireV2 = context -> facet(context).isV2() ? context.proceed() : notFound();

  public final Handler anyVersion = Context::proceed;

  public final Handler search = context -> {
    if (facet(context).isV2()) {
      return notFound();
    }
    Content content = facet(context).search(context, false);
    return content != null ? HttpResponses.ok(content) : notFound();
  };

  public final Handler autocomplete = context -> {
    if (facet(context).isV2()) {
      return notFound();
    }
    Content content = facet(context).search(context, true);
    return content != null ? HttpResponses.ok(content) : notFound();
  };

  public final Handler v2PassThrough = context -> {
    if (!facet(context).isV2()) {
      return notFound();
    }
    Content content = facet(context).passThroughV2(context);
    return content != null ? HttpResponses.ok(content) : notFound();
  };
}
