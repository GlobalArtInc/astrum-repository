
package org.globalart.astrum.internal.node.datastore;

import org.globalart.astrum.datastore.api.SingletonDataAccess;

/**
 * DAO for accessing the single stored node id. Not appropriate for clustered environments.
 *
 * @since 3.37
 */
public interface NodeIdDAO
    extends SingletonDataAccess<String>
{
  /**
   * Attempts to create the singleton nodeId, may fail with constraint violation if this has already been set.
   */
  void create(String nodeId);
}
