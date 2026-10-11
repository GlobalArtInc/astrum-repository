
package org.globalart.astrum.internal.security.apikey;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.ToIntFunction;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.internal.security.apikey.store.ApiKeyStore;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.KeyValueEvent;
import org.globalart.astrum.security.UserPrincipalsExpired;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.apikey.ApiKey;
import org.globalart.astrum.security.authc.apikey.ApiKeyFactory;
import org.globalart.astrum.security.authz.ScopedPrincipal;
import org.globalart.astrum.security.user.UserNotFoundException;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.scheduling.CancelableHelper.checkCancellation;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@ManagedLifecycle(phase = TASKS)
@Component
public class ApiKeyServiceImpl
    extends StateGuardLifecycleSupport
    implements ApiKeyInternalService, EventAware
{
  public static final String MIGRATION_COMPLETE = "astrum.upgrade.api.secrets";

  private final ApiKeyStore apiKeyStore;

  private final ApiKeyStore apiKeyStoreV2;

  private final DatabaseCheck databaseCheck;

  private final GlobalKeyValueStore kv;

  private final UserPrincipalsHelper principalsHelper;

  private final Map<String, ApiKeyFactory> apiKeyFactories;

  private final DefaultApiKeyFactory defaultApiKeyFactory;

  private volatile boolean secretMigrationComplete = false;

  private volatile boolean onVersion = false;

  @Autowired
  public ApiKeyServiceImpl(
      @Qualifier("v1") final ApiKeyStore apiKeyStoreV1,
      @Qualifier("v2") final ApiKeyStore apiKeyStoreV2,
      final DatabaseCheck check,
      final GlobalKeyValueStore kv,
      final UserPrincipalsHelper principalsHelper,
      final List<ApiKeyFactory> apiKeyFactoriesList,
      final DefaultApiKeyFactory defaultApiKeyFactory)
  {
    this.apiKeyStore = checkNotNull(apiKeyStoreV1);
    this.apiKeyStoreV2 = checkNotNull(apiKeyStoreV2);
    this.databaseCheck = checkNotNull(check);
    this.kv = checkNotNull(kv);
    this.principalsHelper = checkNotNull(principalsHelper);
    this.apiKeyFactories = QualifierUtil.buildQualifierBeanMap(checkNotNull(apiKeyFactoriesList));
    this.defaultApiKeyFactory = checkNotNull(defaultApiKeyFactory);
  }

  @Override
  protected void doStart() {
    secretMigrationComplete = kv.getBoolean(MIGRATION_COMPLETE).orElse(false);
  }

  @Override
  public Collection<ApiKey> browseAll(final int page, final int pageSize) {
    return callBrowse(store -> store.browseAll(page, pageSize)
        .stream()
        .map(ApiKey.class::cast)
        .collect(Collectors.toList()));
  }

  @Override
  public Collection<ApiKey> browse(final String domain) {
    return callBrowse(store -> store.browse(domain)
        .stream()
        .map(ApiKey.class::cast)
        .collect(Collectors.toList()));
  }

  @Override
  public Collection<ApiKey> browseByCreatedDate(final String domain, final OffsetDateTime date) {
    return callBrowse(store -> store.browseByCreatedDate(domain, date)
        .stream()
        .map(ApiKey.class::cast)
        .collect(Collectors.toList()));
  }

  @Override
  public Collection<ApiKey> browsePaginated(final String domain, final int page, final int pageSize) {
    return callBrowse(store -> store.browsePaginated(domain, page, pageSize)
        .stream()
        .map(ApiKey.class::cast)
        .collect(Collectors.toList()));
  }

  @Override
  public Collection<ApiKey> browseFiltered(
      final String domain,
      final String realm,
      final String username,
      final int skip,
      final int limit)
  {
    return callBrowse(store -> store.browseFiltered(domain, realm, username, skip, limit)
        .stream()
        .map(ApiKey.class::cast)
        .collect(Collectors.toList()));
  }

  @Override
  public int countFiltered(final String domain, final String realm, final String username) {
    return callModify(store -> store.countFiltered(domain, realm, username));
  }

  @Override
  public int count(final String domain) {
    return callModify(store -> store.count(domain));
  }

  @Override
  public char[] createApiKey(final String domain, final PrincipalCollection principals) {
    checkScopeSupported(principals);
    char[] apiKey = makeApiKey(domain);
    try {
      modify(store -> store.persistApiKey(domain, principals, apiKey));
      return apiKey;
    }
    catch (org.globalart.astrum.datastore.api.DuplicateKeyException e) {
      return handleConcurrentCreation(domain, principals, e);
    }
  }

  private char[] handleConcurrentCreation(
      final String domain,
      final PrincipalCollection principals,
      final org.globalart.astrum.datastore.api.DuplicateKeyException cause)
  {
    String migrationStatus = getMigrationStatus();
    log.debug("Concurrent API key creation for domain={}, principals={}, migrationStatus={}",
        domain, principals, migrationStatus);

    Optional<ApiKey> existingKey;
    if (isMigrationComplete()) {
      existingKey = apiKeyStoreV2.getApiKey(domain, principals).map(ApiKey.class::cast);
    }
    else if (!isOnDBVersion()) {
      existingKey = apiKeyStore.getApiKey(domain, principals).map(ApiKey.class::cast);
    }
    else {
      existingKey = apiKeyStoreV2.getApiKey(domain, principals)
          .map(ApiKey.class::cast)
          .or(() -> apiKeyStore.getApiKey(domain, principals).map(ApiKey.class::cast));
    }

    return existingKey.map(ApiKey::getApiKey)
        .orElseThrow(() -> new IllegalStateException(
            "Concurrent API key creation detected but no existing key found for domain=" + domain +
                ", principals=" + principals + ", migrationStatus=" + migrationStatus,
            cause));
  }

  private String getMigrationStatus() {
    if (isMigrationComplete()) {
      return "complete";
    }
    else if (!isOnDBVersion()) {
      return "not-started";
    }
    else {
      return "in-progress";
    }
  }

  @Override
  public int deleteApiKey(final String domain, final PrincipalCollection principals) {
    if (scopeUnsupported(principals)) {
      return 0;
    }
    return callModify(store -> store.deleteApiKey(domain, principals));
  }

  @Override
  public int deleteApiKeys(final OffsetDateTime expiration) {
    return callModify(store -> store.deleteApiKeys(expiration));
  }

  @Override
  public int deleteApiKeys(final String domain, final OffsetDateTime expiration) {
    return callModify(store -> store.deleteApiKeys(domain, expiration));
  }

  @Override
  public int deleteApiKeys(final PrincipalCollection principals) {
    checkCancellation();
    if (scopeUnsupported(principals)) {
      return 0;
    }
    return callModify(store -> store.deleteApiKeys(principals));
  }

  @Override
  public int deleteApiKeys(final String domain) {
    return callModify(store -> store.deleteApiKeys(domain));
  }

  @Override
  public Optional<ApiKey> getApiKey(final String domain, final PrincipalCollection principals) {
    if (scopeUnsupported(principals)) {
      return Optional.empty();
    }
    return find(store -> store.getApiKey(domain, principals)
        .map(ApiKey.class::cast));
  }

  @Override
  public Optional<ApiKey> getApiKeyByToken(final String domain, final char[] apiKey) {
    return find(store -> store.getApiKeyByToken(domain, apiKey)
        .map(ApiKey.class::cast));
  }

  @Override
  public void persistApiKey(
      final String domain,
      final PrincipalCollection principals,
      final char[] apiKey,
      final OffsetDateTime created)
  {
    checkScopeSupported(principals);
    modify(store -> store.persistApiKey(domain, principals, apiKey, created));
  }

  @Override
  public void persistApiKey(
      final String domain,
      final PrincipalCollection principals,
      final char[] apiKey,
      final OffsetDateTime created,
      final String creatorUserId,
      final String creatorRealm)
  {
    checkScopeSupported(principals);
    modify(store -> store.persistApiKey(domain, principals, apiKey, created, creatorUserId, creatorRealm));
  }

  @Override
  public int purgeApiKeys() {
    checkCancellation();

    // Note: we rely on deleteApiKeys to delete from both stores if appropriate
    return StreamSupport.stream(find(ApiKeyStore::browsePrincipals).spliterator(), false)
        .filter(principal -> !userExists(principal))
        .mapToInt(this::deleteApiKeys)
        .sum();
  }

  @Override
  public void updateApiKeyRealm(final ApiKey from, final PrincipalCollection newPrincipal) {
    modify(store -> store.updateApiKey((ApiKeyInternal) from,
        newPrincipal));
  }

  /**
   * An event handler to remove api keys of expired users.
   */
  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserPrincipalsExpired event) {
    final String userId = event.getUserId();
    if (userId != null) {
      deleteApiKeys(new SimplePrincipalCollection(userId, event.getSource()));
    }
    else {
      purgeApiKeys();
    }
  }

  private char[] makeApiKey(final String domain) {
    ApiKeyFactory factory = apiKeyFactories.get(domain);
    if (factory != null) {
      return checkNotNull(factory.makeApiKey());
    }
    return defaultApiKeyFactory.makeApiKey();
  }

  private boolean userExists(final PrincipalCollection principals) {
    try {
      principalsHelper.getUserStatus(principals);
    }
    catch (UserNotFoundException e) {
      log.debug("Stale user found", e);
      return false;
    }
    catch (Exception e) {
      log.warn("Failed to lookup user {}", principals);
    }
    return true;
  }

  @Subscribe
  public void on(final KeyValueEvent event) {
    if (MIGRATION_COMPLETE.equals(event.getKey())) {
      secretMigrationComplete = (boolean) event.getValue();
    }
  }

  private void modify(final Consumer<ApiKeyStore> consumer) {
    if (isMigrationComplete()) {
      // if migration is complete we only need to update the migrated table
      log.trace("modify new table");
      consumer.accept(apiKeyStoreV2);
    }
    else if (!isOnDBVersion()) {
      // ZDU, if db migration has not begun we can safely modify only the old store
      log.trace("modify old table");
      consumer.accept(apiKeyStore);
    }
    else {
      // DB migration has happened, table migration is in an ambiguous state so we try modifications on both
      log.trace("modify both tables");
      consumer.accept(apiKeyStore);
      consumer.accept(apiKeyStoreV2);
    }
  }

  private int callModify(final ToIntFunction<ApiKeyStore> consumer) {
    if (isMigrationComplete()) {
      // if migration is complete we only need to update the migrated table
      log.trace("callModify new table");
      return consumer.applyAsInt(apiKeyStoreV2);
    }
    else if (!isOnDBVersion()) {
      // ZDU, if db migration has not begun we can safely modify only the old store
      log.trace("callModify old table");
      return consumer.applyAsInt(apiKeyStore);
    }
    // DB migration has happened, table migration is in an ambiguous state so we try modifications on both
    log.trace("callModify both tables");
    return Math.max(consumer.applyAsInt(apiKeyStore), consumer.applyAsInt(apiKeyStoreV2));
  }

  private Collection<ApiKey> callBrowse(final Function<ApiKeyStore, Collection<ApiKey>> fn) {
    if (isMigrationComplete()) {
      log.trace("Browsing new table");
      return fn.apply(apiKeyStoreV2);
    }
    // We'll rely on the modify methods to potentially update both tables
    log.trace("Browsing old table");
    return fn.apply(apiKeyStore);
  }

  private <E> E find(final Function<ApiKeyStore, E> fn) {
    if (isMigrationComplete()) {
      // if migration is complete we only need to lookup the migrated table
      log.trace("find on new table");
      return fn.apply(apiKeyStoreV2);
    }

    return fn.apply(apiKeyStore);
  }

  /**
   * Keys of a scoped sign-in (an access token) must keep the scope. Only the migrated store matches keys by all of
   * their principals; the old one matches by user and realm and would hand out the user's unscoped keys.
   */
  private boolean scopeUnsupported(final PrincipalCollection principals) {
    return !isMigrationComplete() && principals != null && !principals.byType(ScopedPrincipal.class).isEmpty();
  }

  private void checkScopeSupported(final PrincipalCollection principals) {
    if (scopeUnsupported(principals)) {
      throw new IllegalStateException(
          "Keys cannot be issued to a scoped sign-in until the API key secrets migration is complete");
    }
  }

  private boolean isOnDBVersion() {
    onVersion = onVersion || databaseCheck.isAtLeast(SecretsService.SECRETS_MIGRATION_VERSION);

    return onVersion;
  }

  @VisibleForTesting
  boolean isMigrationComplete() {
    return secretMigrationComplete;
  }
}
