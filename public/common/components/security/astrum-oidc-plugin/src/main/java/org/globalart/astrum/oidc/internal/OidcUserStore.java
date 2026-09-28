package org.globalart.astrum.oidc.internal;

import java.util.Collection;
import java.util.Collections;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class OidcUserStore
{
  static final String KEY = "astrum.oidc.users";

  private final GlobalKeyValueStore keyValueStore;

  private Map<String, OidcUser> users;

  @Autowired
  public OidcUserStore(final GlobalKeyValueStore keyValueStore) {
    this.keyValueStore = checkNotNull(keyValueStore);
  }

  public synchronized Optional<OidcUser> get(final String userId) {
    return Optional.ofNullable(users().get(userId));
  }

  public synchronized Collection<OidcUser> list() {
    return Collections.unmodifiableCollection(new TreeMap<>(users()).values());
  }

  public synchronized boolean put(final OidcUser user) {
    OidcUser previous = users().put(user.userId(), user);
    persist();
    return previous == null || !previous.groups().equals(user.groups());
  }

  public synchronized boolean remove(final String userId) {
    boolean removed = users().remove(userId) != null;
    if (removed) {
      persist();
    }
    return removed;
  }

  private Map<String, OidcUser> users() {
    if (users == null) {
      users = new TreeMap<>(keyValueStore.get(KEY, Users.class).map(Users::users).orElse(Map.of()));
    }
    return users;
  }

  private void persist() {
    keyValueStore.setString(KEY, (Object) new Users(new TreeMap<>(users)));
  }

  @JsonIgnoreProperties(ignoreUnknown = true)
  public record Users(Map<String, OidcUser> users)
  {
    public Users {
      users = users == null ? Map.of() : users;
    }
  }
}
