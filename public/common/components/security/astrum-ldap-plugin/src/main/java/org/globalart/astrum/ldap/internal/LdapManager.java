package org.globalart.astrum.ldap.internal;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;
import java.util.function.LongSupplier;

import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.kv.KeyValueEvent;
import org.globalart.astrum.ldap.internal.LdapDirectory.Authentication;
import org.globalart.astrum.security.authz.AuthorizationConfigurationChanged;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.IncorrectCredentialsException;
import org.apache.shiro.authc.UnknownAccountException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Looks users and groups up across the configured LDAP servers, in order. Results are cached for
 * {@code astrum.ldap.cache.ttl}. A server that fails {@code maxIncidentsCount} times in a row is skipped for
 * {@code connectionRetryDelay} seconds.
 */
@Component
public class LdapManager
    implements EventAware
{
  static final int DEFAULT_SEARCH_LIMIT = 100;

  static final int VERIFY_USER_LIMIT = 20;

  private static final Logger log = LoggerFactory.getLogger(LdapManager.class);

  private final LdapConfigurationStore store;

  private final LdapDirectory directory;

  private final EventManager eventManager;

  private final Cache<String, Optional<LdapUser>> users;

  private final Cache<String, CachedLogin> logins;

  private final Map<String, Health> health = new ConcurrentHashMap<>();

  private final SecureRandom random = new SecureRandom();

  private LongSupplier clock = System::currentTimeMillis;

  @Autowired
  public LdapManager(
      final LdapConfigurationStore store,
      final LdapDirectory directory,
      final EventManager eventManager,
      @Value("${astrum.ldap.cache.ttl:5m}") final Time cacheTtl)
  {
    this.store = checkNotNull(store);
    this.directory = checkNotNull(directory);
    this.eventManager = checkNotNull(eventManager);
    long ttl = Math.max(0, cacheTtl.toMillis());
    this.users = CacheBuilder.newBuilder().maximumSize(10_000).expireAfterWrite(ttl, TimeUnit.MILLISECONDS).build();
    this.logins = CacheBuilder.newBuilder().maximumSize(10_000).expireAfterWrite(ttl, TimeUnit.MILLISECONDS).build();
  }

  @VisibleForTesting
  void setClock(final LongSupplier clock) {
    this.clock = checkNotNull(clock);
  }

  public boolean isConfigured() {
    return store.isConfigured();
  }

  /**
   * Authenticates a user against the first server that knows the user.
   *
   * @throws UnknownAccountException       if no server knows the user
   * @throws IncorrectCredentialsException if the password is wrong
   * @throws AuthenticationException       if the user was not found and a server could not be asked
   */
  public LdapUser authenticate(final String userId, final char[] password) {
    String key = key(userId);
    CachedLogin cached = logins.getIfPresent(key);
    if (cached != null && cached.matches(password)) {
      return cached.user();
    }
    LdapServerException failure = null;
    for (LdapServer server : store.list()) {
      if (!available(server)) {
        failure = skipped(server);
        continue;
      }
      Authentication result;
      try {
        result = directory.authenticate(server, store.password(server), userId, password);
        succeeded(server);
      }
      catch (LdapServerException e) {
        failed(server, e);
        failure = e;
        continue;
      }
      switch (result.outcome()) {
        case AUTHENTICATED -> {
          LdapUser user = result.user();
          users.put(key, Optional.of(user));
          logins.put(key, CachedLogin.of(password, user, random));
          return user;
        }
        case INCORRECT_PASSWORD ->
            throw new IncorrectCredentialsException("Incorrect password for LDAP user " + userId);
        case USER_NOT_FOUND -> {
          // try the next server
        }
      }
    }
    if (failure != null) {
      throw new AuthenticationException("LDAP is unavailable: " + failure.getMessage(), failure);
    }
    throw new UnknownAccountException("LDAP user " + userId + " not found");
  }

  /**
   * @throws LdapServerException if the user was not found and a server could not be asked
   */
  public Optional<LdapUser> getUser(final String userId) {
    if (userId == null || userId.isBlank()) {
      return Optional.empty();
    }
    String key = key(userId);
    Optional<LdapUser> cached = users.getIfPresent(key);
    if (cached != null) {
      return cached;
    }
    LdapServerException failure = null;
    for (LdapServer server : store.list()) {
      if (!available(server)) {
        failure = skipped(server);
        continue;
      }
      try {
        Optional<LdapUser> user = directory.findUser(server, store.password(server), userId);
        succeeded(server);
        if (user.isPresent()) {
          users.put(key, user);
          return user;
        }
      }
      catch (LdapServerException e) {
        failed(server, e);
        failure = e;
      }
    }
    if (failure != null) {
      throw failure;
    }
    users.put(key, Optional.empty());
    return Optional.empty();
  }

  /**
   * Searches every available server; a server that fails is left out of the result.
   *
   * @param prefix the start of the user id, or {@code null} for any user
   * @param limit  the most users to take from each server
   */
  public List<LdapUser> searchUsers(final String prefix, final int limit) {
    Map<String, LdapUser> found = new LinkedHashMap<>();
    for (LdapServer server : store.list()) {
      if (!available(server)) {
        continue;
      }
      try {
        for (LdapUser user : directory.searchUsers(server, store.password(server), prefix, limit)) {
          found.putIfAbsent(key(user.username()), user);
        }
        succeeded(server);
      }
      catch (LdapServerException e) {
        failed(server, e);
      }
    }
    return new ArrayList<>(found.values());
  }

  /**
   * @param query part of the group name, or {@code null} for any group
   * @return group names from every available server, sorted
   */
  public List<String> searchGroups(final String query, final int limit) {
    TreeSet<String> groups = new TreeSet<>(String.CASE_INSENSITIVE_ORDER);
    for (LdapServer server : store.list()) {
      if (!available(server) || !LdapDirectory.mapsGroups(server)) {
        continue;
      }
      try {
        groups.addAll(directory.searchGroups(server, store.password(server), query, limit));
        succeeded(server);
      }
      catch (LdapServerException e) {
        failed(server, e);
      }
    }
    return groups.stream().limit(limit > 0 ? limit : Long.MAX_VALUE).toList();
  }

  /**
   * @throws LdapServerException if the group was not found and a server could not be asked
   */
  public boolean groupExists(final String groupId) {
    LdapServerException failure = null;
    for (LdapServer server : store.list()) {
      if (!LdapDirectory.mapsGroups(server)) {
        continue;
      }
      if (!available(server)) {
        failure = skipped(server);
        continue;
      }
      try {
        boolean exists = directory.groupExists(server, store.password(server), groupId);
        succeeded(server);
        if (exists) {
          return true;
        }
      }
      catch (LdapServerException e) {
        failed(server, e);
        failure = e;
      }
    }
    if (failure != null) {
      throw failure;
    }
    return false;
  }

  public void verifyConnection(final LdapServer server, final char[] bindPassword) {
    directory.verifyConnection(server, bindPassword);
  }

  public List<LdapUser> verifyUserMapping(final LdapServer server, final char[] bindPassword) {
    return directory.searchUsers(server, bindPassword, null, VERIFY_USER_LIMIT);
  }

  /**
   * @throws LdapServerException if the user is not found or the password is wrong
   */
  public LdapUser verifyLogin(
      final LdapServer server,
      final char[] bindPassword,
      final String username,
      final char[] password)
  {
    Authentication result = directory.authenticate(server, bindPassword, username, password);
    return switch (result.outcome()) {
      case AUTHENTICATED -> result.user();
      case INCORRECT_PASSWORD -> throw new LdapServerException("The password for user '" + username + "' is incorrect");
      case USER_NOT_FOUND -> throw new LdapServerException("User '" + username + "' was not found");
    };
  }

  /**
   * Forgets cached users, logins and server failures, and has every realm recompute permissions.
   */
  public void clearCache() {
    invalidate();
    eventManager.post(new AuthorizationConfigurationChanged());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final KeyValueEvent event) {
    if (LdapConfigurationStore.KEY.equals(event.getKey())) {
      clearCache();
    }
  }

  private void invalidate() {
    users.invalidateAll();
    logins.invalidateAll();
    health.clear();
  }

  private boolean available(final LdapServer server) {
    Health state = health.get(server.getId());
    return state == null || state.skipUntil() <= clock.getAsLong();
  }

  private void succeeded(final LdapServer server) {
    health.remove(server.getId());
  }

  private void failed(final LdapServer server, final LdapServerException e) {
    log.warn("LDAP server {} failed: {}", server.getName(), e.getMessage());
    log.debug("LDAP server {} failure", server.getName(), e);
    long now = clock.getAsLong();
    health.compute(server.getId(), (id, previous) -> {
      int failures = (previous == null || previous.skipUntil() > 0 ? 0 : previous.failures()) + 1;
      if (server.getMaxIncidentsCount() > 0 && server.getConnectionRetryDelaySeconds() > 0
          && failures >= server.getMaxIncidentsCount()) {
        log.warn("Skipping LDAP server {} for {} seconds after {} failures", server.getName(),
            server.getConnectionRetryDelaySeconds(), failures);
        return new Health(0, now + server.getConnectionRetryDelaySeconds() * 1000L);
      }
      return new Health(failures, 0);
    });
  }

  private static LdapServerException skipped(final LdapServer server) {
    return new LdapServerException("LDAP server " + server.getName() + " is skipped after repeated failures");
  }

  private static String key(final String userId) {
    return userId.toLowerCase(Locale.ENGLISH);
  }

  private record Health(int failures, long skipUntil)
  {
  }

  /**
   * A successful login, kept as a salted hash of the password.
   */
  private record CachedLogin(byte[] salt, byte[] hash, LdapUser user)
  {
    static CachedLogin of(final char[] password, final LdapUser user, final SecureRandom random) {
      byte[] salt = new byte[16];
      random.nextBytes(salt);
      return new CachedLogin(salt, hash(salt, password), user);
    }

    boolean matches(final char[] password) {
      return password != null && password.length > 0 && MessageDigest.isEqual(hash, hash(salt, password));
    }

    private static byte[] hash(final byte[] salt, final char[] password) {
      try {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        digest.update(salt);
        byte[] bytes = LdapPasswords.utf8(password);
        digest.update(bytes);
        Arrays.fill(bytes, (byte) 0);
        return digest.digest();
      }
      catch (NoSuchAlgorithmException e) {
        throw new IllegalStateException(e);
      }
    }
  }
}
