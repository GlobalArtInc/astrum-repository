package org.globalart.astrum.ldap.internal;

import java.util.List;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Posted on the node where an administrator changed the LDAP server list. Other nodes pick the change up from the
 * key-value store.
 */
public class LdapConfigurationChangedEvent
{
  public enum Type
  {
    CREATED,
    UPDATED,
    DELETED,
    REORDERED
  }

  private final Type type;

  private final LdapServer server;

  private final List<String> order;

  private LdapConfigurationChangedEvent(final Type type, final LdapServer server, final List<String> order) {
    this.type = checkNotNull(type);
    this.server = server;
    this.order = order;
  }

  public static LdapConfigurationChangedEvent of(final Type type, final LdapServer server) {
    return new LdapConfigurationChangedEvent(type, checkNotNull(server).copy(), List.of());
  }

  public static LdapConfigurationChangedEvent reordered(final List<String> names) {
    return new LdapConfigurationChangedEvent(Type.REORDERED, null, List.copyOf(names));
  }

  public Type getType() {
    return type;
  }

  /**
   * @return the server that changed; {@code null} for {@link Type#REORDERED}
   */
  public LdapServer getServer() {
    return server;
  }

  /**
   * @return the server names in their new order, for {@link Type#REORDERED}
   */
  public List<String> getOrder() {
    return order;
  }
}
