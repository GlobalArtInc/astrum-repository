
package org.globalart.astrum.repository.maven.rest;

import java.util.Objects;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.ContentDispositionHelper;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * @since 3.20
 */
@Component
public class MavenProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<MavenProxyRepositoryApiRequest>
{
  private static final String MAVEN = "maven";

  private RepositoryManager repositoryManager;

  @Autowired
  public MavenProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Autowired
  public void setRepositoryManager(final RepositoryManager repositoryManager) {
    this.repositoryManager = repositoryManager;
  }

  @Override
  public Configuration convert(final MavenProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    configuration.attributes(MAVEN).set("versionPolicy", request.getMaven().getVersionPolicy());
    configuration.attributes(MAVEN).set("layoutPolicy", request.getMaven().getLayoutPolicy());

    String contentDisposition = ContentDispositionHelper.resolveContentDisposition(
        request.getMaven().getContentDisposition(),
        repositoryManager,
        request.getName(),
        MAVEN);

    configuration.attributes(MAVEN).set("contentDisposition", contentDisposition);

    NestedAttributesMap httpclient = configuration.attributes("httpclient");
    if (Objects.nonNull(httpclient.get("authentication"))) {
      httpclient.child("authentication").set("preemptive", request.getHttpClient().getAuthentication().isPreemptive());
    }
    return configuration;
  }
}
