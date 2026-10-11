package org.globalart.astrum.security.accesstoken;

import java.util.Locale;
import java.util.Set;

import org.apache.shiro.authz.Permission;
import org.apache.shiro.authz.permission.WildcardPermission;

/**
 * What a client signed in with an access token may do.
 */
public enum AccessTokenScope
{
  /**
   * Read and browse only: download artifacts, pull images, search.
   */
  READ,

  /**
   * Everything the owner of the token may do.
   */
  WRITE;

  private static final Set<String> READ_ACTIONS = Set.of("read", "browse");

  public boolean permits(final Permission permission) {
    if (this == WRITE) {
      return true;
    }
    if (!(permission instanceof WildcardPermission)) {
      return false;
    }
    String value = permission.toString().replace("[", "").replace("]", "").replace(" ", "");
    int actionsStart = value.lastIndexOf(':');
    if (actionsStart < 0 || actionsStart == value.length() - 1) {
      return false;
    }
    for (String action : value.substring(actionsStart + 1).split(",")) {
      if (!READ_ACTIONS.contains(action.toLowerCase(Locale.ROOT))) {
        return false;
      }
    }
    return true;
  }

  public String id() {
    return name().toLowerCase(Locale.ROOT);
  }

  public static AccessTokenScope fromId(final String id) {
    for (AccessTokenScope scope : values()) {
      if (scope.id().equalsIgnoreCase(id)) {
        return scope;
      }
    }
    throw new IllegalArgumentException("Unknown access token scope: " + id);
  }
}
