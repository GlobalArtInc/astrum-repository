package org.globalart.astrum.repository.docker.internal.proxy;

import java.util.List;
import java.util.TreeSet;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.docker.internal.v2.DockerMetadataHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.proxy.ProxyFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class DockerProxyTagsHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(DockerProxyTagsHandler.class);

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    DockerRoute route = DockerRoute.get(context);
    List<String> local = DockerMetadataHandler.localTags(context, route.getName());
    List<String> remote = null;
    try {
      remote = ((DockerProxyFacet) context.getRepository().facet(ProxyFacet.class)).remoteTags(context);
    }
    catch (Exception e) {
      log.debug("Unable to list remote tags of {} in {}", route.getName(), context.getRepository().getName(), e);
    }
    if (local == null && remote == null) {
      return DockerMetadataHandler.tags(context, route.getName(), null);
    }
    TreeSet<String> tags = new TreeSet<>();
    if (local != null) {
      tags.addAll(local);
    }
    if (remote != null) {
      tags.addAll(remote);
    }
    return DockerMetadataHandler.tags(context, route.getName(), tags);
  }
}
