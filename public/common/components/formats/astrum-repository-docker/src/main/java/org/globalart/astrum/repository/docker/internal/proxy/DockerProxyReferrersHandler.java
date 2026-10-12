package org.globalart.astrum.repository.docker.internal.proxy;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.internal.v2.DockerDigest;
import org.globalart.astrum.repository.docker.internal.v2.DockerMetadataHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerReferrers;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.proxy.ProxyFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Referrers of a proxy: what the remote reports together with the referrers the proxy has cached, so signatures and
 * SBOMs already pulled through the proxy stay discoverable while the remote is unreachable.
 */
@Component
public class DockerProxyReferrersHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(DockerProxyReferrersHandler.class);

  private final long maxManifestSize;

  public DockerProxyReferrersHandler(@Value("${astrum.docker.manifest.maxSize:4194304}") final long maxManifestSize) {
    this.maxManifestSize = maxManifestSize;
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    DockerRoute route = DockerRoute.get(context);
    if (!DockerDigest.isDigest(route.getReference())) {
      return DockerMetadataHandler.invalidReferrersDigest(route);
    }
    List<List<Map<String, Object>>> lists = new ArrayList<>();
    try {
      List<Map<String, Object>> remote =
          ((DockerProxyFacet) context.getRepository().facet(ProxyFacet.class)).remoteReferrers(context);
      if (remote != null) {
        lists.add(remote);
      }
    }
    catch (Exception e) {
      log.debug("Unable to list remote referrers of {} in {}", route.getReference(),
          context.getRepository().getName(), e);
    }
    DockerContentFacet content = context.getRepository().facet(DockerContentFacet.class);
    lists.add(DockerReferrers.local(content, route.getName(), route.getReference()));
    return DockerReferrers.response(context, route.getName(), route.getReference(), DockerReferrers.merge(lists),
        maxManifestSize);
  }
}
