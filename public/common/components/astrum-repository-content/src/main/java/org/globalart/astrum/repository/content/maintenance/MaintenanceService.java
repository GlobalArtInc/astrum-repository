
package org.globalart.astrum.repository.content.maintenance;

import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

/**
 * A service for executing maintenance operations (such as a 'delete') on assets in a repository.
 *
 * @since 3.26
 */
public interface MaintenanceService
{
  /**
   * Delete an asset in the specified repository.
   *
   * @return Set of asset names that were removed
   */
  Set<String> deleteAsset(Repository repository, Asset asset);

  /**
   * Delete a component in the specified repository.
   *
   * @return Set of asset names that were removed
   */
  Set<String> deleteComponent(Repository repository, Component component);

  /**
   * Delete a path in the specified repository.
   */
  void deleteFolder(Repository repository, String path);

  /**
   * Check if a component can be deleted.
   *
   * @return true if the component can be deleted, false otherwise
   */
  boolean canDeleteComponent(Repository repository, Component component);

  /**
   * Check if an asset can be deleted.
   *
   * @return true if the asset can be deleted, false otherwise
   */
  boolean canDeleteAsset(Repository repository, Asset asset);

  /**
   * Check if user can potentially delete any asset in a given path.
   *
   * @return true if the path can be deleted, false otherwise
   */
  boolean canDeleteFolder(Repository repository, String path);

  /**
   * Deletes a list of assets from the corresponding repository.
   *
   * @param repository Repository to delete assets from
   * @param assetIds List of internal asset IDs to delete
   * @return Set of assets that were removed
   */
  Set<String> deleteAssets(Repository repository, List<Integer> assetIds);
}
