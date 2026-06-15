
package org.globalart.astrum.coreui;

/**
 * Interface for transforming or replacing one or more attributes of an {@link AssetXO}.
 */
public interface AssetAttributeTransformer
{
  /**
   * Transforms the given {@link AssetXO} by modifying or replacing its attributes.
   *
   * @param assetXO the asset to be transformed
   */
  void transform(AssetXO assetXO);
}
