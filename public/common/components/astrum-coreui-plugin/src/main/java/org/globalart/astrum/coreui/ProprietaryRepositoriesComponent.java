
package org.globalart.astrum.coreui;

import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.AuthorizingRepositoryManager;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.validation.Validate;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.config.ConfigurationConstants.COMPONENT;
import static org.globalart.astrum.repository.config.ConfigurationConstants.PROPRIETARY_COMPONENTS;
import org.springframework.stereotype.Component;

/**
 * Proprietary Repositories Settings {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_ProprietaryRepositories")
public class ProprietaryRepositoriesComponent
    extends DirectComponentSupport
{
  /**
   * Formats that do not support namespace confusion protection via Firewall
   */
  private static final Set<String> UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS = Set.of("docker");

  private final AuthorizingRepositoryManager repositoryManager;

  @Autowired
  public ProprietaryRepositoriesComponent(final AuthorizingRepositoryManager authorizingRepositoryManager) {
    this.repositoryManager = checkNotNull(authorizingRepositoryManager);
  }

  /**
   * Retrieves the proprietary repo info, containing a list of all repos with proprietary components
   *
   * @return the list of proprietary repos
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public ProprietaryRepositoriesXO read() {
    List<String> enabledRepositories = repositoryManager.getRepositoriesWithAdmin()
        .stream()
        .filter(ProprietaryRepositoriesComponent::isProprietary)

        .filter(ProprietaryRepositoriesComponent::supportsNamespaceConfusion)
        .map(Repository::getName)
        .sorted()
        .collect(Collectors.toList()); // NOSONAR
    ProprietaryRepositoriesXO proprietaryRepositoriesXO = new ProprietaryRepositoriesXO();
    proprietaryRepositoriesXO.setEnabledRepositories(enabledRepositories);
    return proprietaryRepositoriesXO;
  }

  /**
   * Retrieves hosted repos.
   *
   * @return a list of hosted repo
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:settings:read")
  public List<ReferenceXO> readPossibleRepos() {
    return repositoryManager.getRepositoriesWithAdmin()
        .stream()
        .filter(ProprietaryRepositoriesComponent::isHosted)
        .filter(ProprietaryRepositoriesComponent::supportsNamespaceConfusion)
        .map(repo -> new ReferenceXO(repo.getName(), repo.getName()))
        .sorted(Comparator.comparing(ReferenceXO::getName))
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Updates proprietary repo settings.
   *
   * @return the list of proprietary repos
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  @Validate
  ProprietaryRepositoriesXO update(@NotNull @Valid final ProprietaryRepositoriesXO proprietaryRepositoriesXO) {
    Set<String> shouldBeEnabled = new HashSet<>(proprietaryRepositoriesXO.getEnabledRepositories());
    repositoryManager.getRepositoriesWithAdmin()
        .stream()
        .filter(ProprietaryRepositoriesComponent::isHosted)
        .filter(ProprietaryRepositoriesComponent::supportsNamespaceConfusion)
        .filter(repo -> (!isProprietary(repo) && shouldBeEnabled.contains(repo.getName()))
            || (isProprietary(repo) && !shouldBeEnabled.contains(repo.getName())))
        .forEach(repo -> setProprietaryStatus(repo, shouldBeEnabled.contains(repo.getName())));
    return read();
  }

  private static boolean isHosted(final Repository repository) {
    return HostedType.NAME.equals(repository.getType().getValue());
  }

  private static boolean supportsNamespaceConfusion(final Repository repository) {
    return !UNSUPPORTED_NAMESPACE_CONFUSION_FORMATS.contains(repository.getFormat().getValue());
  }

  private static boolean isProprietary(final Repository repository) {
    return Boolean.TRUE.equals(
        repository.getConfiguration().attributes(COMPONENT).get(PROPRIETARY_COMPONENTS, Boolean.class));
  }

  private void setProprietaryStatus(final Repository repository, final boolean status) {
    Configuration newConfig = repository.getConfiguration().copy();
    newConfig.attributes(COMPONENT).set(PROPRIETARY_COMPONENTS, status);
    try {
      repositoryManager.update(newConfig);
    }
    catch (Exception e) {
      log.error("Error updating proprietary status in repo: {}", repository.getName(), e);
      throw new RuntimeException(e);
    }
  }
}
