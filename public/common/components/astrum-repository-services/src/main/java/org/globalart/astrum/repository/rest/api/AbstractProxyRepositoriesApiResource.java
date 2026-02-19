
package org.globalart.astrum.repository.rest.api;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.BeanParam;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.ProxyRepositoryApiRequest;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;

/**
 * @since 3.28
 */
public class AbstractProxyRepositoriesApiResource<T extends ProxyRepositoryApiRequest>
    extends AbstractRepositoriesApiResource<T>
{
  protected ProxyRepositoryApiRequestToConfigurationConverter<T> configurationConverter;

  @Autowired
  public void setConfigurationConverter(
      final ProxyRepositoryApiRequestToConfigurationConverter<T> configurationConverter)
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
