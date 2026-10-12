package org.globalart.astrum.repository.yum.internal.task;

import java.io.IOException;
import java.io.UncheckedIOException;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.hosted.YumHostedFacet;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AutomatedYumMetadataRebuildTask
    extends RepositoryTaskSupport
{
  public AutomatedYumMetadataRebuildTask() {
    super(false);
  }

  @Override
  public String getMessage() {
    return "Automated Yum metadata rebuild";
  }

  @Override
  protected void execute(final Repository repository) {
    long start = System.currentTimeMillis();
    try {
      repository.facet(YumHostedFacet.class).rebuildMetadata(false);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
    log.debug("Rebuilt the Yum metadata of repository {} in {} ms", repository.getName(),
        System.currentTimeMillis() - start);
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return YumFormat.NAME.equals(repository.getFormat().getValue())
        && HostedType.NAME.equals(repository.getType().getValue());
  }
}
