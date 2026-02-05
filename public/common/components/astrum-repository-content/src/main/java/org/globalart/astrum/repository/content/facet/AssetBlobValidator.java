
package org.globalart.astrum.repository.content.facet;

import javax.annotation.Nullable;

import org.globalart.astrum.common.io.InputStreamSupplier;

/**
 * Something that determines and validates the Content-Type of asset-blobs.
 *
 * @since 3.24
 */
public interface AssetBlobValidator
{
  /**
   * Determines the Content-Type and optionally validates it against the declared Content-Type.
   *
   * @param strictValidation whether the check should be strict or not.
   * @param contentSupplier the supplier of the content to determine or confirm content type.
   * @param assetPath blob name, usually a file path or file name or just extension (file extension is used to
   *          determine content type along with "magic" detection where actual content bits are used,
   *          like file headers or magic bytes). Is optional, but be aware that if present it improves
   *          content type detection reliability.
   * @param declaredContentType if non-null, the declared content type will be confirmed, if null, this method will
   *          attempt to determine the content type.
   * @return the validated Content-Type.
   */
  String determineContentType(
      boolean strictValidation,
      InputStreamSupplier contentSupplier,
      @Nullable String assetPath,
      @Nullable String declaredContentType);
}
