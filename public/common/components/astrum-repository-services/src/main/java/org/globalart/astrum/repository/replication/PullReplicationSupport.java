
package org.globalart.astrum.repository.replication;

import java.util.List;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;

/**
 * Contains any per-format customizations needed for pull replication. Not all formats will need this.
 */
public interface PullReplicationSupport
{
  String IS_REPLICATION_REQUEST = "IS_REPLICATION_REQUEST";

  List<String> translatePaths(final Repository repository, final List<String> paths);

  static boolean isReplicationRequest(final Context context) {
    if (context != null && context.getAttributes() != null) {
      return Boolean.TRUE.equals(context.getAttributes().get(IS_REPLICATION_REQUEST, Boolean.class, false));
    }
    return false;
  }
}
