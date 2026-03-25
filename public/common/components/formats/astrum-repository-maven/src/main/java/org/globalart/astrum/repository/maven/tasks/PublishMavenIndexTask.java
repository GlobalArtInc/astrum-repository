
package org.globalart.astrum.repository.maven.tasks;

import java.io.IOException;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.maven.MavenIndexFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Maven 2 publish MI indexes task.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PublishMavenIndexTask
    extends RepositoryTaskSupport
{
  @Override
  protected void execute(final Repository repository) {
    MavenIndexFacet mavenIndexFacet = repository.facet(MavenIndexFacet.class);
    try {
      mavenIndexFacet.publishIndex();
    }
    catch (IOException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return repository.getFormat().getValue().equals(Maven2Format.NAME);
  }

  @Override
  public String getMessage() {
    return "Publish Maven indexes of " + getRepositoryField();
  }
}
