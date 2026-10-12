package org.globalart.astrum.repository.docker.internal;

import java.util.Set;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

/**
 * Deletes Docker content without leaving referrers behind.
 *
 * Components are tags: deleting a component, or a cleanup policy, removes tags and never the manifest a tag points
 * at, so a subject stays reachable by its digest together with the signatures, SBOMs and attestations attached to it.
 * Deleting a manifest itself removes everything attached to it: the manifests whose {@code subject} it is, the tags of
 * the tag schema derived from its digest, and the tags that point at it, as a delete by digest through the registry
 * API does.
 */
@org.springframework.stereotype.Component
@Qualifier(DockerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DockerMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteAsset(final Asset asset) {
    if (DockerContentFacet.KIND_MANIFEST.equals(asset.kind())) {
      FluentAsset manifest = contentFacet().assets().with(asset);
      String name = DockerContentFacet.nameOf(manifest);
      String digest = DockerContentFacet.digestOf(manifest);
      if (name != null && digest != null && facet(DockerContentFacet.class).deleteManifest(name, digest)) {
        return Set.of(asset.path());
      }
    }
    return super.deleteAsset(asset);
  }
}
