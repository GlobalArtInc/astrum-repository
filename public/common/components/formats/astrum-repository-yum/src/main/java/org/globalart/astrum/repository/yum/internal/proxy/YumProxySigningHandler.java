package org.globalart.astrum.repository.yum.internal.proxy;

import java.io.InputStream;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.yum.internal.YumPaths;
import org.globalart.astrum.repository.yum.internal.YumSigningFacet;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.internal.YumPaths.requestPath;

@Component
public class YumProxySigningHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    YumSigningFacet signing = context.getRepository().facet(YumSigningFacet.class);
    if (!signing.isConfigured()) {
      return context.proceed();
    }
    String path = requestPath(context);
    if (YumPaths.PUBLIC_KEY.equals(YumPaths.fileName(path))) {
      return signing.publicKey().map(HttpResponses::ok).orElseGet(HttpResponses::notFound);
    }
    Content repomd = context.getRepository().facet(YumProxyFacet.class)
        .repomd(context, YumPaths.baseOfMetadata(path).orElse(""));
    if (repomd == null) {
      return HttpResponses.notFound();
    }
    byte[] bytes;
    try (InputStream in = repomd.openInputStream()) {
      bytes = in.readAllBytes();
    }
    return signing.sign(bytes)
        .map(signature -> HttpResponses.ok(new BytesPayload(signature, YumSigningFacet.SIGNATURE_CONTENT_TYPE)))
        .orElseGet(HttpResponses::notFound);
  }
}
