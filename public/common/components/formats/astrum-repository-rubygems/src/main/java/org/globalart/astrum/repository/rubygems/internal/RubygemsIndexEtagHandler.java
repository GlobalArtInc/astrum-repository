package org.globalart.astrum.repository.rubygems.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.springframework.stereotype.Component;

@Component
public class RubygemsIndexEtagHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Response response = context.proceed();
    if (response.getStatus().isSuccessful() && response.getPayload() instanceof Content content) {
      RubygemsResponses.withMd5Etag(content);
    }
    return response;
  }
}
