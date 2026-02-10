
package org.globalart.astrum.repository.export;

import org.globalart.astrum.repository.content.fluent.FluentAsset;

/**
 * Used to determine {@link FluentAsset} specifics during export.
 */
public interface ExportAssetFilter
{
  /**
   * Determines whether the export of the asset should be skipped for the provided {@link FluentAsset}
   *
   * @param asset the {@link FluentAsset} to use in the decision
   * @return true if the asset export should be skipped
   */
  boolean shouldSkipAsset(FluentAsset asset);

  /**
   * Determines whether the export of the attributes should be skipped for the provided {@link FluentAsset}
   *
   * @param asset the {@link FluentAsset} to use in the decision
   * @return true if the attribute export should be skipped
   */
  default boolean shouldSkipAttributes(FluentAsset asset) {
    return false;
  }

  /**
   * Provides a calculated export path for the provided {@link FluentAsset}
   *
   * @param asset the {@link FluentAsset} to calculate the export path for
   * @return the calculated export path
   */
  default String getAssetExportPath(FluentAsset asset) {
    return asset.path();
  }
}
