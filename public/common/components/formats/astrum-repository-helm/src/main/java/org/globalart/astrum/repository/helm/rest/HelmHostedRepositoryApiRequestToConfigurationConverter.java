package org.globalart.astrum.repository.helm.rest;

import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

@Component
public class HelmHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<HelmHostedRepositoryApiRequest>
{}
