
package org.globalart.astrum.node.datastore;

import java.util.Optional;

import org.globalart.astrum.common.node.NodeAccess;

/**
 * Store for accessing the node id in single node environments.
 *
 * @since 3.37
 */
public interface NodeIdStore
{
  /**
   * Remove the currently persisted node id, this will not change the {@link NodeAccess}.
   */
  void clear();

  /**
   * Retrieve the current node id if it exists.
   *
   * @return the node id
   */
  Optional<String> get();

  /**
   * Retrieve the current node id if it exists, otherwise create & store a new one.
   *
   * @return the node id
   */
  String getOrCreate();

  /**
   * Set the current node id, this will not update the {@link NodeAccess}
   * 
   * @param nodeId
   */
  void set(final String nodeId);
}
