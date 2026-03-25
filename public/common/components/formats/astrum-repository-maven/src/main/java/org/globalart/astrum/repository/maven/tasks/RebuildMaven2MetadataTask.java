
package org.globalart.astrum.repository.maven.tasks;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.maven.MavenMetadataRebuildFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.scheduling.Cancelable;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.maven.tasks.RebuildMaven2MetadataTaskDescriptor.ARTIFACTID_FIELD_ID;
import static org.globalart.astrum.repository.maven.tasks.RebuildMaven2MetadataTaskDescriptor.BASEVERSION_FIELD_ID;
import static org.globalart.astrum.repository.maven.tasks.RebuildMaven2MetadataTaskDescriptor.CASCADE_REBUILD;
import static org.globalart.astrum.repository.maven.tasks.RebuildMaven2MetadataTaskDescriptor.GROUPID_FIELD_ID;
import static org.globalart.astrum.repository.maven.tasks.RebuildMaven2MetadataTaskDescriptor.REBUILD_CHECKSUMS;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven 2 metadata rebuild task.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RebuildMaven2MetadataTask
    extends RepositoryTaskSupport
    implements Cancelable
{

  private final Type hostedType;

  private final Format maven2Format;

  @Autowired
  public RebuildMaven2MetadataTask(
      @Qualifier(HostedType.NAME) final Type hostedType,
      @Qualifier(Maven2Format.NAME) final Format maven2Format)
  {
    this.hostedType = checkNotNull(hostedType);
    this.maven2Format = checkNotNull(maven2Format);
  }

  @Override
  protected void execute(final Repository repository) {
    MavenMetadataRebuildFacet mavenHostedFacet = repository.facet(MavenMetadataRebuildFacet.class);
    mavenHostedFacet.rebuildMetadata(
        getConfiguration().getString(GROUPID_FIELD_ID),
        getConfiguration().getString(ARTIFACTID_FIELD_ID),
        getConfiguration().getString(BASEVERSION_FIELD_ID),
        getConfiguration().getBoolean(REBUILD_CHECKSUMS, false),
        getConfiguration().getBoolean(CASCADE_REBUILD, true),
        false);
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return maven2Format.equals(repository.getFormat()) && hostedType.equals(repository.getType());
  }

  @Override
  public String getMessage() {
    return "Rebuilding Maven Metadata of " + getRepositoryField();
  }

}
