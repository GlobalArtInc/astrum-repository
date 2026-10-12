
package org.globalart.astrum.repository.huggingface.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.huggingface.rest.HuggingFaceProxyRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Component
@Path(RESOURCE_URI)
public class HuggingFaceProxyRepositoriesApiResourceV1
    extends HuggingFaceProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/huggingface/proxy";

  @Autowired
  public void setConfigurationConverter(
      final HuggingFaceProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
