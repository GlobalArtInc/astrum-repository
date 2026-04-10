
package org.globalart.astrum.repository.apt.datastore.internal.metadata;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.List;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Base class for automated APT metadata rebuild tasks.
 * Provides common implementation for both hosted and proxy repository types.
 * <p>
 * These tasks are triggered automatically by the metadata scheduler in response to repository events
 * such as asset creation, deletion, or updates. They invoke the metadata facet to rebuild repository
 * metadata (Package indexes, Release files, signatures).
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AutomatedAptMetadataRebuildTask
    extends RepositoryTaskSupport
{
  protected AutomatedAptMetadataRebuildTask() {
    super(false);
  }

  @Override
  public String getMessage() {
    return "Automated APT metadata rebuild";
  }

  @Override
  protected void execute(final Repository repository) {
    log.info("Starting automated APT metadata rebuild for repository {}", repository.getName());

    try {
      long start = System.currentTimeMillis();

      // Delegate to the appropriate metadata facet (hosted or proxy)
      rebuildMetadata(repository);

      long duration = System.currentTimeMillis() - start;
      log.info("Completed APT metadata rebuild for repository {} in {} ms", repository.getName(), duration);
    }
    catch (IOException e) {
      log.error("Failed to rebuild APT metadata for repository {}", repository.getName(), e);
      throw new UncheckedIOException(e);
    }
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return List.of(ProxyType.NAME, HostedType.NAME).contains(repository.getType().getValue()) &&
        AptFormat.NAME.equals(repository.getFormat().getValue());
  }

  /**
   * Rebuilds metadata for the given repository by invoking the appropriate metadata facet.
   * <p>
   * Hosted implementation: Invokes
   * {@link org.globalart.astrum.repository.apt.datastore.internal.hosted.metadata.AptHostedMetadataFacet#rebuildMetadata()}
   * <br>
   * Proxy implementation: Invokes
   * {@link org.globalart.astrum.repository.apt.datastore.internal.proxy.metadata.AptProxyMetadataFacet#rebuildMetadata()}
   *
   * @param repository the repository to rebuild metadata for
   * @throws IOException if metadata rebuild fails
   */
  protected void rebuildMetadata(final Repository repository) throws IOException {
    repository.facet(AptMetadataFacetSupport.class).rebuildMetadata();
  }
}
