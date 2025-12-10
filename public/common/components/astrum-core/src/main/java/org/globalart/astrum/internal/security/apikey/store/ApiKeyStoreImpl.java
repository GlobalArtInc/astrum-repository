
package org.globalart.astrum.internal.security.apikey.store;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.Collections;
import java.util.Optional;
import java.util.Set;
import java.util.function.Predicate;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.internal.security.apikey.ApiKeyInternal;
import org.globalart.astrum.transaction.Transactional;

import org.apache.shiro.subject.PrincipalCollection;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Legacy {@link ApiKeyStore} implementation
 *
 * @since 3.21
 */
@Deprecated
@Component
@Qualifier("v1")
public class ApiKeyStoreImpl
    extends ConfigStoreSupport<ApiKeyDAO>
    implements ApiKeyStore, EventAware
{
  @Autowired
  public ApiKeyStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  @Override
  public void persistApiKey(
      final String domain,
      final PrincipalCollection principals,
      final char[] apiKey,
      final OffsetDateTime created)
  {
    ApiKeyData apiKeyData = new ApiKeyData();
    apiKeyData.setDomain(domain);
    apiKeyData.setPrincipals(principals);
    apiKeyData.setApiKey(apiKey);
    apiKeyData.setCreated(created);
    dao().save(apiKeyData);
  }

  @Transactional
  @Override
  public Optional<ApiKeyInternal> getApiKey(
      final String domain,
      final PrincipalCollection principals)
  {
    return findApiKey(domain, principals);
  }

  @Transactional
  @Override
  public Optional<ApiKeyInternal> getApiKeyByToken(
      final String domain,
      final char[] apiKey)
  {
    return dao().findPrincipals(domain, new ApiKeyToken(apiKey));
  }

  @Transactional
  @Override
  public int deleteApiKey(final String domain, final PrincipalCollection principals) {
    return findApiKey(domain, principals)
        .map(ApiKeyInternal::getApiKey)
        .map(ApiKeyToken::new)
        .map(token -> dao().deleteKey(domain, token))
        .orElse(0);
  }

  @Transactional
  @Override
  public int deleteApiKeys(final PrincipalCollection principals) {
    return dao().findApiKeysForPrimary(principals.getPrimaryPrincipal().toString())
        .stream()
        .filter(principalMatches(principals))
        .map(key -> dao().deleteKey(key.getDomain(), new ApiKeyToken(key.getApiKey())))
        .mapToInt(Integer::intValue)
        .sum();
  }

  @Override
  @Transactional
  public Iterable<PrincipalCollection> browsePrincipals() {
    return dao().browsePrincipals();
  }

  @Transactional
  @Override
  public Collection<ApiKeyInternal> browse(final String domain) {
    return dao().browse(domain);
  }

  @Transactional
  @Override
  public Collection<ApiKeyInternal> browseByCreatedDate(final String domain, final OffsetDateTime date) {
    return dao().browseByCreatedDate(domain, date);
  }

  @Transactional
  @Override
  public int count(final String domain) {
    return dao().count(domain);
  }

  @Transactional
  @Override
  public int deleteApiKeys(final String domain) {
    return dao().deleteApiKeysByDomain(domain);
  }

  @Transactional
  @Override
  public int deleteApiKeys(final OffsetDateTime expiration) {
    return dao().deleteApiKeyByExpirationDate(expiration);
  }

  @Transactional
  @Override
  public int deleteApiKeys(final String domain, final OffsetDateTime expiration) {
    return dao().deleteApiKeysByDomainAndExpiration(domain, expiration);
  }

  @Transactional
  @Override
  public void updateApiKey(
      final ApiKeyInternal from,
      final PrincipalCollection principalCollection)
  {
    dao().update(
        new ApiKeyData(from.getDomain(), principalCollection, new ApiKeyToken(from.getApiKey()), from.getCreated()));
  }

  @Override
  public Collection<ApiKeyInternal> browseAll(final int page, final int pageSize) {
    // This deprecated V1 store does not support bulk pagination; it always returns an empty list.
    // API key export requires secrets migration to be complete so that keys are served by the V2 store.
    return Collections.emptyList();
  }

  @Transactional
  @Override
  public Collection<ApiKeyInternal> browsePaginated(
      final String domain,
      final int page,
      final int pageSize)
  {
    return dao().browsePaginated(domain, (page - 1) * pageSize, pageSize);
  }

  @Transactional
  public Collection<ApiKeyData> browseAllSince(final OffsetDateTime last, final int pageSize) {
    return dao().browseAllSince(last, pageSize);
  }

  /*
   * Finds ApiKey records for the provided username, and ensures the realm is the same
   */
  private Optional<ApiKeyInternal> findApiKey(final String domain, final PrincipalCollection principals) {
    return dao().findApiKeys(domain, principals.getPrimaryPrincipal().toString())
        .stream()
        .filter(principalMatches(principals))
        .findAny();
  }

  /*
   * Creates a Predicate which ensures the principal of the tested ApiKey is equal to the provided PrincipalCollection
   */
  private Predicate<ApiKeyInternal> principalMatches(final PrincipalCollection principals) {
    String primaryPrincipal = principals.getPrimaryPrincipal().toString();
    Set<String> realms = principals.getRealmNames();
    return key -> key.getPrincipals().getRealmNames().equals(realms)
        && key.getPrincipals().getPrimaryPrincipal().equals(primaryPrincipal);
  }

  @Override
  public Collection<ApiKeyInternal> browseFiltered(
      final String domain,
      final String realm,
      final String username,
      final int skip,
      final int limit)
  {
    // V1 store doesn't support SQL filtering, fall back to browse + filter
    return browse(domain).stream()
        .filter(key -> username == null || username.equals(key.getPrincipals().getPrimaryPrincipal()))
        .skip(skip)
        .limit(limit)
        .collect(Collectors.toList());
  }

  @Override
  public int countFiltered(final String domain, final String realm, final String username) {
    // V1 store doesn't support SQL filtering, fall back to browse + count
    return (int) browse(domain).stream()
        .filter(key -> username == null || username.equals(key.getPrincipals().getPrimaryPrincipal()))
        .count();
  }
}
