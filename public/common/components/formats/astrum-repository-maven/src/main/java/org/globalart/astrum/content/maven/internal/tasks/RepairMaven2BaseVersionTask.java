
package org.globalart.astrum.content.maven.internal.tasks;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.content.maven.store.Maven2ComponentData;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.scheduling.Cancelable;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RepairMaven2BaseVersionTask
    extends RepositoryTaskSupport
    implements Cancelable
{
  @Override
  protected void execute(final Repository repository) {
    MavenContentFacet mavenContentFacet = repository.facet(MavenContentFacet.class);
    Iterable<FluentComponent> componentsWithMissedBaseVersion = mavenContentFacet.getComponentsWithMissedBaseVersion();
    for (FluentComponent fluentComponent : componentsWithMissedBaseVersion) {
      Maven2ComponentData componentData = new Maven2ComponentData();
      componentData.setNamespace(fluentComponent.namespace());
      componentData.setName(fluentComponent.name());
      componentData.setVersion(fluentComponent.version());
      componentData.setRepositoryId(mavenContentFacet.contentRepositoryId());
      NestedAttributesMap maven2 = fluentComponent.attributes("maven2");
      componentData.setBaseVersion(maven2.get("baseVersion", String.class));
      mavenContentFacet.updateBaseVersion(componentData);
    }
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return repository.getFormat().getValue().equals(Maven2Format.NAME);
  }

  @Override
  public String getMessage() {
    return "Fixed Maven Base Versions of " + getRepositoryField();
  }
}
