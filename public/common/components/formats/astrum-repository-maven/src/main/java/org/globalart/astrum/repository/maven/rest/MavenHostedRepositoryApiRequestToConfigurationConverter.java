
package org.globalart.astrum.repository.maven.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.ContentDispositionHelper;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * @since 3.20
 */
@Component
public class MavenHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<MavenHostedRepositoryApiRequest>
{
  private static final String MAVEN = "maven";

  private RepositoryManager repositoryManager;

  @Autowired
  public void setRepositoryManager(final RepositoryManager repositoryManager) {
    this.repositoryManager = repositoryManager;
  }

  @Override
  public Configuration convert(final MavenHostedRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    configuration.attributes(MAVEN).set("versionPolicy", request.getMaven().getVersionPolicy());
    configuration.attributes(MAVEN).set("layoutPolicy", request.getMaven().getLayoutPolicy());

    String contentDisposition = ContentDispositionHelper.resolveContentDisposition(
        request.getMaven().getContentDisposition(),
        repositoryManager,
        request.getName(),
        MAVEN);

    configuration.attributes(MAVEN).set("contentDisposition", contentDisposition);

    return configuration;
  }
}
