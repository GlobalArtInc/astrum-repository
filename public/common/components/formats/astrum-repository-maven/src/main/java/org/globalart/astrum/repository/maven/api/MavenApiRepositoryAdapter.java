
package org.globalart.astrum.repository.maven.api;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.maven.ContentDisposition;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.rest.HttpClientAttributesWithPreemptiveAuth;
import org.globalart.astrum.repository.maven.rest.HttpClientConnectionAuthenticationAttributesWithPreemptive;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Adapter to expose maven specific repository configuration for the repositories REST API.
 *
 * @since 3.20
 */
@Component
@Qualifier(Maven2Format.NAME)
public class MavenApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  private static final String MAVEN = "maven";

  @Autowired
  public MavenApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    boolean online = repository.getConfiguration().isOnline();
    String name = repository.getName();
    String url = repository.getUrl();

    switch (repository.getType().toString()) {
      case HostedType.NAME:
        return new MavenHostedApiRepository(
            name,
            url,
            online,
            getHostedStorageAttributes(repository),
            getCleanupPolicyAttributes(repository),
            createMavenAttributes(repository),
            getComponentAttributes(repository));
      case ProxyType.NAME:
        return new MavenProxyApiRepository(name, url, online,
            getHostedStorageAttributes(repository),
            getCleanupPolicyAttributes(repository),
            getProxyAttributes(repository),
            getNegativeCacheAttributes(repository),
            getHttpClientAttributes(repository),
            getRoutingRuleName(repository),
            createMavenAttributes(repository),
            getReplicationAttributes(repository));
      default:
        return super.adapt(repository);
    }
  }

  private MavenAttributes createMavenAttributes(final Repository repository) {
    String versionPolicy = repository.getConfiguration().attributes(MAVEN).get("versionPolicy", String.class);
    String layoutPolicy = repository.getConfiguration().attributes(MAVEN).get("layoutPolicy", String.class);
    String contentDisposition = repository.getConfiguration().attributes(MAVEN).get("contentDisposition", String.class);
    // Normalize null to INLINE - legacy repos with null serve inline at handler level
    if (contentDisposition == null) {
      contentDisposition = ContentDisposition.INLINE.name();
    }
    return new MavenAttributes(versionPolicy, layoutPolicy, contentDisposition);
  }

  @Override
  protected HttpClientAttributesWithPreemptiveAuth getHttpClientAttributes(final Repository repository) {
    HttpClientAttributes httpClientAttributes = super.getHttpClientAttributes(repository);
    HttpClientConnectionAuthenticationAttributesWithPreemptive authentication = null;

    Configuration configuration = repository.getConfiguration();
    NestedAttributesMap httpclient = configuration.attributes("httpclient");
    if (httpclient.contains("authentication")) {
      NestedAttributesMap authenticationMap = httpclient.child("authentication");
      Boolean preemptive = authenticationMap.get("preemptive", Boolean.class);

      authentication =
          new HttpClientConnectionAuthenticationAttributesWithPreemptive(httpClientAttributes.getAuthentication(),
              preemptive);
    }

    return new HttpClientAttributesWithPreemptiveAuth(httpClientAttributes, authentication);
  }
}
