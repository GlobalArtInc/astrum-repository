package org.globalart.astrum.accesstoken.internal;

import java.security.SecureRandom;
import java.time.Clock;
import java.time.Duration;
import java.time.OffsetDateTime;
import java.time.temporal.ChronoUnit;
import java.util.Collection;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.TimeUnit;

import javax.annotation.Nullable;

import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.accesstoken.AccessTokenPrincipal;
import org.globalart.astrum.security.accesstoken.AccessTokenScope;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.user.UserDeletedEvent;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import com.google.common.hash.Hashing;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;

/**
 * Issues, checks and revokes access tokens.
 *
 * A token is shown once, when it is created; only its SHA-256 hash is stored.
 */
@Component
public class AccessTokenService
    implements EventAware
{
  public static final String PREFIX = "astrum_";

  static final int SECRET_LENGTH = 40;

  static final int MAX_NAME_LENGTH = 100;

  static final int MAX_TOKENS_PER_USER = 100;

  static final int MAX_EXPIRATION_DAYS = 3650;

  private static final String ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

  private static final int HINT_LENGTH = 4;

  private static final Duration LAST_USED_RESOLUTION = Duration.ofMinutes(5);

  private static final Logger log = LoggerFactory.getLogger(AccessTokenService.class);

  private final AccessTokenStore store;

  private final EventManager eventManager;

  private final ApiKeyService apiKeyService;

  private final SecureRandom random = new SecureRandom();

  private final Cache<String, Optional<AccessTokenData>> tokensByHash = CacheBuilder.newBuilder()
      .expireAfterWrite(30, TimeUnit.SECONDS)
      .maximumSize(10_000)
      .build();

  private Clock clock = Clock.systemUTC();

  @Autowired
  public AccessTokenService(
      final AccessTokenStore store,
      final EventManager eventManager,
      final ApiKeyService apiKeyService)
  {
    this.store = checkNotNull(store);
    this.eventManager = checkNotNull(eventManager);
    this.apiKeyService = checkNotNull(apiKeyService);
  }

  @VisibleForTesting
  void setClock(final Clock clock) {
    this.clock = checkNotNull(clock);
  }

  public static boolean looksLikeToken(@Nullable final char[] password) {
    if (password == null || password.length != PREFIX.length() + SECRET_LENGTH) {
      return false;
    }
    for (int i = 0; i < PREFIX.length(); i++) {
      if (password[i] != PREFIX.charAt(i)) {
        return false;
      }
    }
    return true;
  }

  public CreatedAccessToken create(
      final PrincipalCollection owner,
      final String name,
      final AccessTokenScope scope,
      @Nullable final Integer expiresInDays)
  {
    checkNotNull(owner);
    checkNotNull(scope);
    String username = owner.getPrimaryPrincipal().toString();
    String trimmedName = name == null ? "" : name.trim();
    if (trimmedName.isEmpty() || trimmedName.length() > MAX_NAME_LENGTH) {
      throw new AccessTokenException("Token name must be 1 to " + MAX_NAME_LENGTH + " characters long");
    }
    if (expiresInDays != null && (expiresInDays < 1 || expiresInDays > MAX_EXPIRATION_DAYS)) {
      throw new AccessTokenException("A token must expire in 1 to " + MAX_EXPIRATION_DAYS + " days");
    }
    if (store.exists(username, trimmedName)) {
      throw new AccessTokenException("You already have a token named '" + trimmedName + "'");
    }
    if (store.browseByUser(username).size() >= MAX_TOKENS_PER_USER) {
      throw new AccessTokenException("You can have at most " + MAX_TOKENS_PER_USER + " tokens");
    }

    String secret = generateSecret();
    OffsetDateTime now = now();

    AccessTokenData token = new AccessTokenData();
    token.setId(UUID.randomUUID().toString());
    token.setUsername(username);
    token.setPrincipals(storablePrincipals(owner));
    token.setName(trimmedName);
    token.setTokenHash(hash(secret));
    token.setTokenHint(secret.substring(0, PREFIX.length() + HINT_LENGTH));
    token.setScope(scope.id());
    token.setCreated(now);
    token.setExpires(expiresInDays == null ? null : now.plusDays(expiresInDays));
    store.create(token);

    log.info("Created access token '{}' for user {}", trimmedName, username);
    eventManager.post(new AccessTokenEvent(AccessTokenEvent.Type.CREATED, token));
    return new CreatedAccessToken(token, secret);
  }

  public Collection<AccessTokenData> list(final String username) {
    return store.browseByUser(username);
  }

  public Collection<AccessTokenData> listAll() {
    return store.browse();
  }

  public Optional<AccessTokenData> read(final String id) {
    return store.read(id);
  }

  /**
   * The principals of a request signed in with this token: its owner's, narrowed to the token's scope.
   */
  public static PrincipalCollection principals(final AccessTokenData token) {
    PrincipalCollection owner = token.getPrincipals();
    SimplePrincipalCollection principals = new SimplePrincipalCollection(owner);
    principals.add(new AccessTokenPrincipal(token.getId(), token.getName(), token.getScope()),
        owner.getRealmNames().iterator().next());
    return principals;
  }

  public boolean revoke(final AccessTokenData token) {
    boolean deleted = store.delete(token.getId());
    tokensByHash.invalidate(token.getTokenHash());
    try {
      apiKeyService.deleteApiKeys(principals(token));
    }
    catch (RuntimeException e) {
      log.warn("Could not delete keys issued through access token {}", token.getId(), e);
    }
    if (deleted) {
      log.info("Revoked access token '{}' of user {}", token.getName(), token.getUsername());
      eventManager.post(new AccessTokenEvent(AccessTokenEvent.Type.REVOKED, token));
    }
    return deleted;
  }

  /**
   * Returns the token with this secret if it exists and has not expired, and records that it was used.
   */
  public Optional<AccessTokenData> authenticate(final char[] secret) {
    if (!looksLikeToken(secret)) {
      return Optional.empty();
    }
    String tokenHash = hash(new String(secret));
    Optional<AccessTokenData> token = tokensByHash.getIfPresent(tokenHash);
    if (token == null) {
      token = store.findByHash(tokenHash);
      tokensByHash.put(tokenHash, token);
    }
    OffsetDateTime now = now();
    if (token.isEmpty() || token.get().isExpired(now)) {
      return Optional.empty();
    }
    markUsed(token.get(), now);
    return token;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserDeletedEvent event) {
    String username = event.getUser().getUserId();
    int deleted = store.deleteByUser(username);
    if (deleted > 0) {
      log.info("Deleted {} access tokens of removed user {}", deleted, username);
      tokensByHash.invalidateAll();
    }
  }

  private void markUsed(final AccessTokenData token, final OffsetDateTime now) {
    OffsetDateTime lastUsed = token.getLastUsed();
    if (lastUsed != null && lastUsed.plus(LAST_USED_RESOLUTION).isAfter(now)) {
      return;
    }
    token.setLastUsed(now);
    try {
      store.updateLastUsed(token.getId(), now);
    }
    catch (RuntimeException e) {
      log.debug("Could not record use of access token {}", token.getId(), e);
    }
  }

  /**
   * The owner's principals without anything but plain names, which is all the store can keep.
   */
  private static PrincipalCollection storablePrincipals(final PrincipalCollection owner) {
    SimplePrincipalCollection principals = new SimplePrincipalCollection();
    for (String realmName : owner.getRealmNames()) {
      for (Object principal : owner.fromRealm(realmName)) {
        if (principal instanceof String) {
          principals.add(principal, realmName);
        }
      }
    }
    return principals;
  }

  private OffsetDateTime now() {
    return OffsetDateTime.now(clock).truncatedTo(ChronoUnit.MILLIS);
  }

  private String generateSecret() {
    StringBuilder secret = new StringBuilder(PREFIX.length() + SECRET_LENGTH).append(PREFIX);
    for (int i = 0; i < SECRET_LENGTH; i++) {
      secret.append(ALPHABET.charAt(random.nextInt(ALPHABET.length())));
    }
    return secret.toString();
  }

  static String hash(final String secret) {
    return Hashing.sha256().hashString(secret, UTF_8).toString();
  }

  public record CreatedAccessToken(AccessTokenData token, String secret)
  {
  }
}
