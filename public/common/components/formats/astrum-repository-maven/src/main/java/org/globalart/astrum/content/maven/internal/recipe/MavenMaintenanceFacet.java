
package org.globalart.astrum.content.maven.internal.recipe;

import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import com.google.common.collect.Sets;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Scope;

import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * @since 3.29
 */
@Qualifier(Maven2Format.NAME)
@Scope(SCOPE_PROTOTYPE)
@org.springframework.stereotype.Component
public class MavenMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    return Sets.union(super.deleteComponent(component), mavenContentFacet().deleteMetadataOrFlagForRebuild(component));
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    return mavenContentFacet().deleteComponents(components);
  }

  private MavenContentFacet mavenContentFacet() {
    return facet(MavenContentFacet.class);
  }
}
