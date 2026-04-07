package org.globalart.astrum.repository.docker.internal.proxy;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.v2.DockerContentResponses;
import org.globalart.astrum.repository.docker.internal.v2.DockerError;
import org.globalart.astrum.repository.docker.internal.v2.DockerLocations;
import org.globalart.astrum.repository.docker.internal.v2.DockerResponses;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DockerProxyHandler
    extends ProxyHandler
{
  @Nullable
  @Override
  protected Response buildMethodNotAllowedResponse(final Context context) {
    String action = context.getRequest().getAction();
    if (!HttpMethods.GET.equals(action) && !HttpMethods.HEAD.equals(action)) {
      return DockerResponses.error(DockerError.UNSUPPORTED,
          "repository " + context.getRepository().getName() + " is a proxy and does not accept " + action);
    }
    return null;
  }

  @Override
  protected Response buildPayloadResponse(final Context context, final Payload payload) {
    DockerRoute route = DockerRoute.get(context);
    if (route.getType() == DockerRoute.Type.BLOB) {
      return DockerContentResponses.blob(payload, route.getReference());
    }
    return DockerContentResponses.manifest(payload, route.getReference());
  }

  @Override
  protected Response buildNotFoundResponse(final Context context) {
    DockerRoute route = DockerRoute.get(context);
    if (route.getType() == DockerRoute.Type.BLOB) {
      return DockerResponses.error(DockerError.BLOB_UNKNOWN, DockerError.BLOB_UNKNOWN.message(),
          Map.of("digest", route.getReference()));
    }
    return DockerResponses.error(DockerError.MANIFEST_UNKNOWN, DockerError.MANIFEST_UNKNOWN.message(),
        Map.of("name", DockerLocations.externalName(context, route.getName()), "reference", route.getReference()));
  }
}
