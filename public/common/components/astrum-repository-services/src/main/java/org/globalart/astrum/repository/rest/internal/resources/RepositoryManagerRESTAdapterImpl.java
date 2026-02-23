
package org.globalart.astrum.repository.rest.internal.resources;

import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.WebApplicationException;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.rest.api.RepositoryMetricsDTO;
import org.globalart.astrum.repository.rest.api.RepositoryMetricsService;
import org.globalart.astrum.repository.rest.api.RepositoryXO;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;
import org.globalart.astrum.repository.types.ProxyType;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptyMap;
import static java.util.Collections.singletonMap;
import static java.util.Optional.ofNullable;
import static java.util.stream.Collectors.toMap;
import static org.apache.commons.lang3.StringUtils.EMPTY;
import static org.globalart.astrum.repository.http.HttpStatus.FORBIDDEN;
import static org.globalart.astrum.repository.http.HttpStatus.UNPROCESSABLE_ENTITY;
import org.springframework.stereotype.Component;

/**
 * An implementation of the {@link RepositoryManagerRESTAdapter}
 *
 * @since 3.4
 */
@Component
public class RepositoryManagerRESTAdapterImpl
    implements RepositoryManagerRESTAdapter
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final RepositoryManager repositoryManager;

  private final ConfigurationStore configurationStore;

  private final Map<String, Recipe> recipes;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  private final Optional<RepositoryMetricsService> repositoryMetricsService;

  @Autowired
  public RepositoryManagerRESTAdapterImpl(
      final RepositoryManager repositoryManager,
      final ConfigurationStore configurationStore,
      final List<Recipe> recipesList,
      final RepositoryPermissionChecker repositoryPermissionChecker,
      @Nullable final RepositoryMetricsService repositoryMetricsService)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.configurationStore = checkNotNull(configurationStore);
    this.recipes = QualifierUtil.buildQualifierBeanMap(checkNotNull(recipesList));
    this.repositoryPermissionChecker = checkNotNull(repositoryPermissionChecker);
    this.repositoryMetricsService = Optional.ofNullable(repositoryMetricsService);
  }

  @Override
  public Repository toRepository(final String repositoryId) {
    if (repositoryId == null) {
      throw new WebApplicationException("repositoryId is required.", UNPROCESSABLE_ENTITY);
    }
    return ofNullable(repositoryManager.get(repositoryId))
        .orElseThrow(() -> new NotFoundException("Unable to locate repository with id " + repositoryId));
  }

  @Override
  public Repository getRepository(final String repositoryId) {
    Repository repository = toRepository(repositoryId);

    if (repositoryPermissionChecker.userCanReadOrBrowse(repository)) {
      // browse or read implies complete access to the repository.
      return repository;
    }
    else {
      // repository exists but user does not have the appropriate permission to browse, return a 403
      throw new WebApplicationException(FORBIDDEN);
    }
  }

  @Override
  public Repository getReadableRepository(String repositoryId) {
    if (repositoryId == null) {
      throw new WebApplicationException("repositoryId is required.", UNPROCESSABLE_ENTITY);
    }
    Repository repository = ofNullable(repositoryManager.get(repositoryId))
        .orElseThrow(() -> new NotFoundException("Unable to locate repository with id " + repositoryId));

    if (!userCanReadOrBrowseOrGroupPermissions(repository)) {
      // user does not have the appropriate permission to browse, return a 403
      throw new WebApplicationException(FORBIDDEN);
    }

    // browse or read implies complete access to the repository.
    return repository;
  }

  private boolean userCanReadOrBrowseOrGroupPermissions(final Repository repository) {
    // Given - repository = raw-hosted
    // as-repository-view-raw-raw-hosted-read - allowed
    // as-repository-view-raw-raw-group-read(raw-group contains raw-hosted as a member) - allowed
    Set<String> repositories = new HashSet<>(repositoryManager.findContainingGroups(repository.getName()));
    repositories.add(repository.getName());

    return repositories
        .stream()
        .map(repositoryManager::get)
        .filter(Objects::nonNull)
        .anyMatch(repositoryPermissionChecker::userCanReadOrBrowse);
  }

  @Override
  public List<RepositoryXO> getRepositories() {
    Configuration[] configurations = configurationStore.list().toArray(new Configuration[0]);
    Map<String, Long> repositorySizes = repositoryMetricsService.map(s -> s.list()
        .stream()
        .collect(
            toMap(RepositoryMetricsDTO::getName, RepositoryMetricsDTO::getSize)))
        .orElse(null);
    return repositoryPermissionChecker.userCanBrowseRepositories(configurations)
        .stream()
        .map(c -> asRepository(c, repositorySizes))
        .collect(Collectors.toList());
  }

  @Override
  public Set<String> findContainingGroups(final String repositoryName) {
    return repositoryManager.findContainingGroups(repositoryName);
  }

  @Override
  public Optional<Long> getRepositorySize(String repositoryName) {
    return repositoryMetricsService.flatMap(s -> s.get(repositoryName).map(d -> d.totalSize));
  }

  private Type getType(final Configuration configuration) {
    Recipe recipe = recipes.get(configuration.getRecipeName());
    return recipe.getType();
  }

  private Format getFormat(final Configuration configuration) {
    Recipe recipe = recipes.get(configuration.getRecipeName());
    return recipe.getFormat();
  }

  private static String getUrl(final String repositoryName) {
    return BaseUrlHolder.get() + "/repository/" + repositoryName;
  }

  private Map<String, Object> attributes(final Configuration configuration) {
    if (getType(configuration) instanceof ProxyType) {
      NestedAttributesMap attrs = configuration.attributes("proxy");
      if (attrs != null) {
        Object remoteUrl = attrs.get("remoteUrl", EMPTY);
        return singletonMap("proxy", singletonMap("remoteUrl", remoteUrl));
      }
    }

    return emptyMap();
  }

  private RepositoryXO asRepository(final Configuration configuration, final Map<String, Long> sizeMap) {
    RepositoryXO xo = new RepositoryXO();
    String repositoryName = configuration.getRepositoryName();
    Type type = getType(configuration);

    xo.setName(repositoryName);
    xo.setType(type.getValue());
    xo.setFormat(getFormat(configuration).getValue());
    xo.setUrl(getUrl(repositoryName));
    xo.setOnline(configuration.isOnline());
    xo.setAttributes(attributes(configuration));

    if (sizeMap != null) {
      xo.setSize(sizeMap.getOrDefault(repositoryName, 0L));
    }
    else {
      xo.setSize(0L);
    }

    return xo;
  }

}
