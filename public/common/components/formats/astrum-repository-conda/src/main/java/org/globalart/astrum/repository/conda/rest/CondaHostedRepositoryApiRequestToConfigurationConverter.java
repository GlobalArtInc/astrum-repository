package org.globalart.astrum.repository.conda.rest;

import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

@Component
public class CondaHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<CondaHostedRepositoryApiRequest>
{}
