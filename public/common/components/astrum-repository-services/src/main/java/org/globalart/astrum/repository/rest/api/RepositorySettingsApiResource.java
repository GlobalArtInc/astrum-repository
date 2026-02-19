
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.26
 */
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class RepositorySettingsApiResource
    implements Resource, RepositorySettingsApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AuthorizingRepositoryManager authorizingRepositoryManager;

  private final Map<String, ApiRepositoryAdapter> convertersByFormat;

  private final ApiRepositoryAdapter defaultAdapter;

  @Autowired
  public RepositorySettingsApiResource(
      final AuthorizingRepositoryManager authorizingRepositoryManager,
      @Qualifier("default") final ApiRepositoryAdapter defaultAdapter,
      final List<ApiRepositoryAdapter> convertersByFormatList)
  {
    this.authorizingRepositoryManager = checkNotNull(authorizingRepositoryManager);
    this.defaultAdapter = checkNotNull(defaultAdapter);
    this.convertersByFormat = QualifierUtil.buildQualifierBeanMap(checkNotNull(convertersByFormatList));
  }

  @Override
  @RequiresAuthentication
  @GET
  public List<AbstractApiRepository> getRepositories() {
    return authorizingRepositoryManager.getRepositoriesWithAdmin()
        .stream()
        .map(this::convert)
        .collect(Collectors.toList());
  }

  private AbstractApiRepository convert(final Repository repository) {
    return convertersByFormat.getOrDefault(repository.getFormat().toString(), defaultAdapter)
        .adaptDecorated(repository);
  }
}
