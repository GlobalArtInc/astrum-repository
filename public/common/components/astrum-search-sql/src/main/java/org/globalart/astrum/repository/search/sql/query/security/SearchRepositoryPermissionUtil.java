
package org.globalart.astrum.repository.search.sql.query.security;

import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryViewPermission;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorManager;

import org.apache.commons.lang3.StringUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptyList;
import static java.util.Optional.ofNullable;
import static java.util.stream.Collectors.toList;
import static org.apache.commons.lang3.StringUtils.EMPTY;
import static org.globalart.astrum.security.BreadActions.BROWSE;
import static org.globalart.astrum.selector.SelectorConfiguration.EXPRESSION;
import org.springframework.stereotype.Component;

/**
 * Checks repositories for 'BROWSE' permissions, fetches active content selectors for specified repositories.
 */
@Component
public class SearchRepositoryPermissionUtil
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final RepositoryManager repositoryManager;

  private final SecurityHelper securityHelper;

  private final SelectorManager selectorManager;

  @Autowired
  public SearchRepositoryPermissionUtil(
      final RepositoryManager repositoryManager,
      final SecurityHelper securityHelper,
      final SelectorManager selectorManager)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.securityHelper = checkNotNull(securityHelper);
    this.selectorManager = checkNotNull(selectorManager);
  }

  public Set<String> browsableAndUnknownRepositories(final Set<String> repositories) {
    log.debug("Checking {} for 'BROWSE' permissions and unknown repositories.", repositories);

    Set<String> browsableAndUnknownRepositories = new HashSet<>();

    /*
     * We need to include unknown repositories as well because we'll need to filter the content selectors
     * by repositories the user may have specified in the search request. This is to make sure that
     * applying the generated sql for cross repository/format content selectors privilege doesn't wrongly return results
     * Therefore, for a given user a cross format or cross repository content selector will always be found for that
     * user even if the
     * search request specifies a repository that does not exist. Thus, since content selector expressions don't include
     * repository
     * we additionally apply the reposistories specified in the search request to the content selector.
     */
    for (String repository : repositories) {
      if (!repositoryManager.exists(repository) ||
          securityHelper.allPermitted(new RepositoryViewPermission(repositoryManager.get(repository), BROWSE))) {
        browsableAndUnknownRepositories.add(repository);
      }
    }

    log.debug("Repositories with 'BROWSE' permissions and unknown repositories: {}", browsableAndUnknownRepositories);

    return browsableAndUnknownRepositories;
  }

  public List<SelectorConfiguration> selectorConfigurations(
      final Set<String> repositories)
  {
    if (repositories.isEmpty()) {
      return emptyList();
    }

    log.debug("Fetching content selectors for {}", repositories);
    Map<String, SelectorConfiguration> distinctConfigs = new HashMap<>();

    List<SelectorConfiguration> selectors = selectorManager.browseActive(repositories, Collections.emptyList())
        .stream()
        .filter(selector -> distinctBySelectorExpression(selector, distinctConfigs))
        .collect(toList());

    log.debug("Found {} distinct content selectors.", selectors.size());

    return selectors;
  }

  private boolean distinctBySelectorExpression(
      final SelectorConfiguration selector,
      final Map<String, SelectorConfiguration> selectorConfigs)
  {
    String expression = ofNullable(selector.getAttributes())
        .map(attr -> attr.get(EXPRESSION))
        .filter(StringUtils::isNotBlank)
        .map(StringUtils::trim)
        .orElse(EMPTY);
    return selectorConfigs.putIfAbsent(expression, selector) == null;
  }
}
