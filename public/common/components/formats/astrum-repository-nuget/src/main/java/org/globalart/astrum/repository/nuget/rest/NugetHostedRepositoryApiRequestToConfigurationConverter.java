package org.globalart.astrum.repository.nuget.rest;

import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

@Component
public class NugetHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<NugetHostedRepositoryApiRequest>
{}
