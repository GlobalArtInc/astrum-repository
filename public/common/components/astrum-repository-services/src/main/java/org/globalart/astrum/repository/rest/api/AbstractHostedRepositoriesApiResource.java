
package org.globalart.astrum.repository.rest.api;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.BeanParam;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;

/**
 * @since 3.28
 */
public class AbstractHostedRepositoriesApiResource<T extends HostedRepositoryApiRequest>
    extends AbstractRepositoriesApiResource<T>
{
  protected HostedRepositoryApiRequestToConfigurationConverter<T> configurationConverter;

  @Autowired
  public void setConfigurationConverter(
      final HostedRepositoryApiRequestToConfigurationConverter<T> configurationConverter)
  {
    this.configurationConverter = configurationConverter;
  }

  @Override
  public Configuration adapt(final T request) {
    return configurationConverter.convert(request);
  }

  @GET
  @Path("/{repositoryName}")
  @Operation(summary = "Get repository")
  @Override
  public AbstractApiRepository getRepository(
      @Parameter(hidden = true) @BeanParam final FormatAndType formatAndType,
      @PathParam("repositoryName") final String repositoryName)
  {
    return super.getRepository(formatAndType, repositoryName);
  }
}
