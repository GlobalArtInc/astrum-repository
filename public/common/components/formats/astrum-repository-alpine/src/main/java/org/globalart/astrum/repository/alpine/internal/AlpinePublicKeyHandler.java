package org.globalart.astrum.repository.alpine.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.fileName;
import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.requestPath;

@Component
public class AlpinePublicKeyHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) {
    String keyFileName = fileName(requestPath(context));
    return context.getRepository()
        .optionalFacet(AlpineSigningFacet.class)
        .flatMap(signing -> signing.publicKey(keyFileName))
        .map(HttpResponses::ok)
        .orElseGet(HttpResponses::notFound);
  }
}
