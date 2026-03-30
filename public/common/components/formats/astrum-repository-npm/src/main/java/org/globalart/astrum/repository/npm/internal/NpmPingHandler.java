
package org.globalart.astrum.repository.npm.internal;

import org.springframework.stereotype.Component;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

/**
 * @since 3.21
 */
@Component
public class NpmPingHandler
    implements Handler
{

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception
  {
    StringPayload payload = new StringPayload("{}", APPLICATION_JSON);
    return HttpResponses.ok(payload);
  }
}
