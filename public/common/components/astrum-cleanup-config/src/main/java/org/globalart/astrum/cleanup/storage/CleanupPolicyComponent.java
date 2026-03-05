
package org.globalart.astrum.cleanup.storage;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.extdirect.model.StoreLoadParameters;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryAdminPermission;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Collections.emptyList;
import static java.util.Collections.singletonList;
import static java.util.Optional.ofNullable;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.security.BreadActions.ADD;
import static org.globalart.astrum.security.BreadActions.READ;
import org.springframework.stereotype.Component;

/**
 * Cleanup policies config {@link DirectComponent}.
 *
 * @since 3.14
 */
@Component
@DirectAction(action = "cleanup_CleanupPolicy")
public class CleanupPolicyComponent
    extends DirectComponentSupport
{
  private final CleanupPolicyStorage cleanupPolicyStorage;

  private final RepositoryManager repositoryManager;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  @Autowired
  public CleanupPolicyComponent(
      final CleanupPolicyStorage cleanupPolicyStorage,
      final RepositoryManager repositoryManager,
      final RepositoryPermissionChecker repositoryPermissionChecker)
  {
    this.cleanupPolicyStorage = checkNotNull(cleanupPolicyStorage);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.repositoryPermissionChecker = repositoryPermissionChecker;
  }

  /**
   * Retrieve {@link CleanupPolicy}s by format.
   *
   * @return a list of {@link CleanupPolicy}s
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<CleanupPolicyXO> readByFormat(final StoreLoadParameters parameters) {
    return ofNullable(parameters.getFilter("format"))
        .map(format -> {
          ensureUserHasPermissionToCleanupPolicyByFormat(format);
          return format;
        })
        .map(this::getAllByFormat)
        .orElse(emptyList());
  }

  private List<CleanupPolicyXO> getAllByFormat(final String format) {
    return cleanupPolicyStorage.getAllByFormat(format)
        .stream()
        .map(CleanupPolicyXO::fromCleanupPolicy)
        .collect(toList());
  }

  private void ensureUserHasPermissionToCleanupPolicyByFormat(final String format) {
    RepositoryAdminPermission permission = new RepositoryAdminPermission(format, "*", singletonList(ADD));
    repositoryPermissionChecker.ensureUserHasAnyPermissionOrAdminAccess(
        singletonList(permission),
        READ,
        repositoryManager.browse());
  }
}
