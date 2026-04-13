
package org.globalart.astrum.repository.apt.rest;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.apt.internal.gpg.AptSigningFacet;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.stereotype.Component;

/**
 * @since 3.20
 */
@Component
public class AptProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<AptProxyRepositoryApiRequest>
{
  @Autowired
  public AptProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public Configuration convert(final AptProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    configuration.attributes("apt").set("distribution", request.getApt().getDistribution());
    configuration.attributes("apt").set("flat", request.getApt().getFlat());
    configuration.attributes("apt")
        .set("enforceDistribution",
            Boolean.TRUE.equals(request.getApt().getEnforceDistribution()));

    // Add signing configuration if provided (optional for proxy repos)
    if (request.getAptSigning() != null) {
      if (!Strings2.isBlank(request.getAptSigning().getKeypair())) {
        configuration.attributes(AptSigningFacet.CONFIG_KEY).set("keypair", request.getAptSigning().getKeypair());
      }
      configuration.attributes(AptSigningFacet.CONFIG_KEY).set("passphrase", request.getAptSigning().getPassphrase());
    }

    return configuration;
  }
}
