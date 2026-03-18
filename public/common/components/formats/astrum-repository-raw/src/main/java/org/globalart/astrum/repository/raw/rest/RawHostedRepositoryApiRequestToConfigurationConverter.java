
package org.globalart.astrum.repository.raw.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.rest.api.ContentDispositionHelper;

import static org.globalart.astrum.repository.raw.rest.RawAttributes.CONTENT_DISPOSITION;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * @since 3.25
 */
@Component
public class RawHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<RawHostedRepositoryApiRequest>
{
  private RepositoryManager repositoryManager;

  @Autowired
  public void setRepositoryManager(final RepositoryManager repositoryManager) {
    this.repositoryManager = repositoryManager;
  }

  @Override
  public Configuration convert(final RawHostedRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);

    String requestedDisposition = null;
    if (request.getRaw() != null && request.getRaw().getContentDisposition() != null) {
      requestedDisposition = request.getRaw().getContentDisposition().name();
    }

    String contentDisposition = ContentDispositionHelper.resolveContentDisposition(
        requestedDisposition,
        repositoryManager,
        request.getName(),
        "raw");

    configuration.attributes("raw").set(CONTENT_DISPOSITION, contentDisposition);
    return configuration;
  }
}
