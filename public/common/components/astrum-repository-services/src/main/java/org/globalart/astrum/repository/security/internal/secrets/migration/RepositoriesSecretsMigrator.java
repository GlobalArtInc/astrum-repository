
package org.globalart.astrum.repository.security.internal.secrets.migration;

import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.security.secrets.SecretMigrationException;
import org.globalart.astrum.security.secrets.SecretsMigratorSupport;

import com.google.common.annotations.VisibleForTesting;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RepositoriesSecretsMigrator
    extends SecretsMigratorSupport
{
  @VisibleForTesting
  static final String HTTP_CLIENT_KEY = "httpclient";

  @VisibleForTesting
  static final String AUTHENTICATION_KEY = "authentication";

  @VisibleForTesting
  static final String PASSWORD_KEY = "password";

  /**
   * Cleanup attribute keys. Duplicated as string literals (rather than importing
   * {@code CleanupPolicyConstants}) because {@code astrum-cleanup-config} depends on this module, so a
   * dependency the other way would create a cycle.
   */
  @VisibleForTesting
  static final String CLEANUP_KEY = "cleanup";

  @VisibleForTesting
  static final String CLEANUP_POLICY_NAME_KEY = "policyName";

  /**
   * Matches the message produced by {@code CleanupConfigurationValidator} when a repository references a
   * cleanup policy that does not exist ({@code "Cleanup Policy '<name>' does not exist."}).
   */
  private static final Pattern MISSING_CLEANUP_POLICY = Pattern.compile("Cleanup Policy '(.+?)' does not exist\\.");

  private final RepositoryManager repositoryManager;

  @Autowired
  public RepositoriesSecretsMigrator(final SecretsService secretsService, final RepositoryManager repositoryManager) {
    super(secretsService);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public void migrate() {
    for (Repository repository : repositoryManager.browse()) {
      CancelableHelper.checkCancellation();

      if (repository.getType() instanceof ProxyType) {
        migrateProxy(repository);
      }
    }
  }

  private void migrateProxy(final Repository repository) {
    Configuration configuration = repository.getConfiguration().copy();
    boolean needUpdate = false;

    // Trim remoteUrl to fix validation errors from legacy data with trailing whitespace (ASTRUM-51248)
    String remoteUrl = configuration.attributes("proxy").get("remoteUrl", String.class);
    if (remoteUrl != null && !remoteUrl.equals(remoteUrl.trim())) {
      configuration.attributes("proxy").set("remoteUrl", remoteUrl.trim());
      needUpdate = true;
    }

    Map<String, Object> authConfig = Optional.ofNullable(configuration.getAttributes())
        .map(global -> global.get(HTTP_CLIENT_KEY))
        .map(http -> (Map<String, Object>) http.get(AUTHENTICATION_KEY))
        .orElse(Collections.emptyMap());

    Secret passwordKey = Optional.ofNullable((String) authConfig.get(PASSWORD_KEY))
        .map(secretsService::from)
        .orElse(null);
    if (passwordKey != null && isLegacyEncryptedString(passwordKey)) {
      needUpdate = true;
      authConfig.put(PASSWORD_KEY, new String(passwordKey.decrypt()));
    }

    if (needUpdate) {
      save(configuration);
    }
  }

  /*
   * Updates a repository configuration, if a failure occurs then secrets will be removed
   */
  private void save(final Configuration configuration) {
    try {
      // repository manager encrypts and handles removal in case of failure
      repositoryManager.update(configuration);
    }
    catch (Exception e) {
      // ASTRUM-53457: a repository may reference a cleanup policy that no longer exists (e.g. the
      // "string" Swagger placeholder or the legacy "None" sentinel). repositoryManager.update() re-runs
      // every ConfigurationValidator and aborts on it, but cleanup-policy validity is irrelevant to
      // secret migration. Strip only the stale reference(s) and retry once so the re-encrypted secret is
      // still persisted and the migration is not blocked for every other repository. Mirrors the
      // in-migrator data sanitization established for remoteUrl in ASTRUM-51248.
      if (removeStaleCleanupPolicies(configuration, e)) {
        try {
          repositoryManager.update(configuration);
          return;
        }
        catch (Exception retryFailure) {
          throw new SecretMigrationException(
              "Failed to migrate repository: " + configuration.getRepositoryName(), retryFailure);
        }
      }
      throw new SecretMigrationException("Failed to migrate repository: " + configuration.getRepositoryName(), e);
    }
  }

  /**
   * If {@code failure} was caused by the repository referencing cleanup policies that do not exist, removes
   * exactly those stale references from {@code configuration} so the update can be retried.
   *
   * @return {@code true} if the configuration was modified (and a retry is worthwhile)
   */
  @SuppressWarnings("unchecked")
  private boolean removeStaleCleanupPolicies(final Configuration configuration, final Exception failure) {
    Set<String> missingPolicies = extractMissingCleanupPolicies(failure);
    if (missingPolicies.isEmpty()) {
      return false;
    }

    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    Map<String, Object> cleanup = attributes != null ? attributes.get(CLEANUP_KEY) : null;
    if (cleanup == null) {
      return false;
    }

    Collection<String> policyNames = (Collection<String>) cleanup.get(CLEANUP_POLICY_NAME_KEY);
    List<String> retained = policyNames == null
        ? Collections.emptyList()
        : policyNames.stream().filter(name -> !missingPolicies.contains(name)).collect(Collectors.toList());

    if (policyNames != null && retained.size() == policyNames.size()) {
      // none of the missing policies were actually referenced here; nothing we can fix
      return false;
    }

    if (retained.isEmpty()) {
      attributes.remove(CLEANUP_KEY);
    }
    else {
      cleanup.put(CLEANUP_POLICY_NAME_KEY, retained);
    }

    log.warn("Repository '{}' references non-existent cleanup policies {}; removing the stale reference(s) so "
        + "secret migration can proceed (ASTRUM-53457).", configuration.getRepositoryName(), missingPolicies);
    return true;
  }

  private static Set<String> extractMissingCleanupPolicies(final Exception failure) {
    if (!(failure instanceof ConstraintViolationException)) {
      return Collections.emptySet();
    }
    return ((ConstraintViolationException) failure).getConstraintViolations()
        .stream()
        .map(ConstraintViolation::getMessage)
        .filter(Objects::nonNull)
        .flatMap(message -> MISSING_CLEANUP_POLICY.matcher(message).results().map(result -> result.group(1)))
        .collect(Collectors.toSet());
  }
}
