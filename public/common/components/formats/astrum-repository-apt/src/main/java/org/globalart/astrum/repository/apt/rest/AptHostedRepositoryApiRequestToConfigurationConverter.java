
package org.globalart.astrum.repository.apt.rest;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

/**
 * @since 3.20
 */
@Component
public class AptHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<AptHostedRepositoryApiRequest>
{
  @Override
  public Configuration convert(final AptHostedRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    configuration.attributes("apt").set("distribution", request.getApt().getDistribution());
    if (request.getAptSigning() != null) {
      if (!Strings2.isBlank(request.getAptSigning().getKeypair())) {
        configuration.attributes("aptSigning").set("keypair", request.getAptSigning().getKeypair());
      }
      configuration.attributes("aptSigning").set("passphrase", request.getAptSigning().getPassphrase());
    }
    return configuration;
  }
}
