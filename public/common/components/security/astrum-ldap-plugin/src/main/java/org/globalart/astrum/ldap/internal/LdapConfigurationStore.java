package org.globalart.astrum.ldap.internal;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.KeyValueEvent;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Keeps the ordered list of LDAP servers in the key-value store, and their bind passwords in the secrets store.
 */
@Component
public class LdapConfigurationStore
    implements EventAware
{
  static final String KEY = "astrum.ldap.servers";

  static final String SECRET_PURPOSE = "ldap";

  private static final Logger log = LoggerFactory.getLogger(LdapConfigurationStore.class);

  private final GlobalKeyValueStore keyValueStore;

  private final SecretsService secretsService;

  private volatile List<LdapServer> cached;

  @Autowired
  public LdapConfigurationStore(final GlobalKeyValueStore keyValueStore, final SecretsService secretsService) {
    this.keyValueStore = checkNotNull(keyValueStore);
    this.secretsService = checkNotNull(secretsService);
  }

  /**
   * @return copies of the configured servers, in the order they are tried
   */
  public List<LdapServer> list() {
    List<LdapServer> current = cached;
    if (current == null) {
      current = keyValueStore.get(KEY, Servers.class)
          .map(Servers::getServers)
          .map(List::copyOf)
          .orElse(List.of());
      cached = current;
    }
    return current.stream().map(LdapServer::copy).toList();
  }

  public boolean isConfigured() {
    return !list().isEmpty();
  }

  public synchronized void save(final List<LdapServer> servers) {
    List<LdapServer> copy = servers.stream().map(LdapServer::copy).toList();
    keyValueStore.setString(KEY, (Object) new Servers(new ArrayList<>(copy)));
    cached = copy;
  }

  public String encryptPassword(final String password, final String userId) {
    return secretsService.encrypt(SECRET_PURPOSE, password.toCharArray(), userId).getId();
  }

  /**
   * @return the decrypted bind password of the server, or {@code null} if it has none
   */
  public char[] password(final LdapServer server) {
    if (server.getAuthPasswordId() == null) {
      return null;
    }
    return secretsService.from(server.getAuthPasswordId()).decrypt();
  }

  public void removePassword(final String secretId) {
    if (secretId == null) {
      return;
    }
    try {
      secretsService.remove(secretsService.from(secretId));
    }
    catch (RuntimeException e) {
      log.warn("Could not remove the LDAP bind password {}", secretId, e);
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final KeyValueEvent event) {
    if (KEY.equals(event.getKey())) {
      cached = null;
    }
  }

  @JsonIgnoreProperties(ignoreUnknown = true)
  public static class Servers
  {
    private List<LdapServer> servers = new ArrayList<>();

    public Servers() {
    }

    public Servers(final List<LdapServer> servers) {
      this.servers = servers;
    }

    public List<LdapServer> getServers() {
      return servers;
    }

    public void setServers(final List<LdapServer> servers) {
      this.servers = servers != null ? servers : new ArrayList<>();
    }
  }
}
