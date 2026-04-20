
package org.globalart.astrum.repository.composer.internal;

import java.io.IOException;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.ComposerHostedFacet;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;

import com.google.common.collect.ImmutableSet;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

@Facet.Exposed
@org.springframework.stereotype.Component
@Qualifier(ComposerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ComposerMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    ImmutableSet.Builder<String> deletedPaths = ImmutableSet.builder();
    deletedPaths.addAll(super.deleteComponent(component));

    String vendor = component.namespace();
    String project = component.name();

    Optional<ComposerHostedFacet> hostedFacet = optionalFacet(ComposerHostedFacet.class);
    if (hostedFacet.isPresent()) {
      try {
        if (!hostedFacet.get().rebuildPackageJson(vendor, project).isPresent()) {
          deletedPaths.add(ComposerPathUtils.buildPackagePath(vendor, project));
        }
        if (!hostedFacet.get().rebuildProviderJson(vendor, project).isPresent()) {
          deletedPaths.add(ComposerPathUtils.buildProviderPath(vendor, project));
        }
      }
      catch (IOException e) {
        log.warn("Failed to rebuild Composer metadata for {}/{} in {}", vendor, project, getRepository().getName(), e);
      }
    }

    return deletedPaths.build();
  }
}
