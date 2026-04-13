
package org.globalart.astrum.repository.apt.internal.gpg;

import org.globalart.astrum.repository.apt.internal.snapshot.AptSnapshotHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.http.HttpMethods.GET;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
public class AptSigningHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Response handle(final Context context) throws Exception {
    String path = assetPath(context);
    String method = context.getRequest().getAction();
    AptSigningFacet facet = context.getRepository().facet(AptSigningFacet.class);

    if ("repository-key.gpg".equals(path) && GET.equals(method)) {
      return HttpResponses.ok(facet.getPublicKey());
    }

    return context.proceed();
  }

  private String assetPath(final Context context) {
    return context.getAttributes().require(AptSnapshotHandler.State.class).assetPath;
  }
}
