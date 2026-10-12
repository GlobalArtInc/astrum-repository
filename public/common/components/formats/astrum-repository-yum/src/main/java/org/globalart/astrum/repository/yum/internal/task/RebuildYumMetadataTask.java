package org.globalart.astrum.repository.yum.internal.task;

import java.io.IOException;
import java.io.UncheckedIOException;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.hosted.YumHostedFacet;
import org.globalart.astrum.scheduling.Cancelable;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RebuildYumMetadataTask
    extends RepositoryTaskSupport
    implements Cancelable
{
  @Override
  public String getMessage() {
    return "Rebuild the Yum metadata of " + getRepositoryField();
  }

  @Override
  protected void execute(final Repository repository) {
    boolean soft = getConfiguration().getBoolean(RebuildYumMetadataTaskDescriptor.SOFT_REPAIR_FIELD_ID, false);
    log.info("Rebuilding the Yum metadata of repository {} ({})", repository.getName(),
        soft ? "changed directories only" : "every directory");
    try {
      repository.facet(YumHostedFacet.class).rebuildMetadata(!soft);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return YumFormat.NAME.equals(repository.getFormat().getValue())
        && HostedType.NAME.equals(repository.getType().getValue());
  }
}
