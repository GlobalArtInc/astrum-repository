
package org.globalart.astrum.common.node;

import java.util.List;
import java.util.Optional;

/**
 * persistence mechanism for {@link NodeConfiguration}
 *
 * @since 3.6.1
 */
public interface NodeConfigurationSource
{
  /**
   * Retrieves all node configuration records
   *
   * @return {@link List} of {@link NodeConfiguration} entities
   */
  List<NodeConfiguration> loadAll();

  /**
   * Retrieves node configuration record by node identity, if the record exists
   *
   * @param nodeId node identifier
   * @return {@link Optional} of {@link NodeConfiguration}
   */
  Optional<NodeConfiguration> getById(String nodeId);

  /**
   * Creates a node configuration record, given a {@link NodeConfiguration}
   *
   * @param configuration {@link NodeConfiguration} entity
   * @return {@link String} node identity
   */
  String create(NodeConfiguration configuration);

  /**
   * Updates the node configuration record of the entity with a matching nodeId
   *
   * @param configuration {@link NodeConfiguration} entity
   * @return whether update succeeded
   */
  boolean update(NodeConfiguration configuration);

  /**
   * Deletes the node configuration record of the entity with the matching nodeId
   *
   * @param nodeId node identifier
   * @return whether delete succeeded
   */
  boolean delete(String nodeId);

  /**
   * Updates the node configuration record of the entity with a matching nodeId to have the given friendly name
   *
   * @param nodeId node identifier
   * @param friendlyName specified by admin
   */
  void setFriendlyName(String nodeId, String friendlyName);

  /**
   * returns the current node's friendly name
   *
   * @since 3.6.1
   *
   * @return friendlyName specified by admin
   */
  String sayMyName();
}
