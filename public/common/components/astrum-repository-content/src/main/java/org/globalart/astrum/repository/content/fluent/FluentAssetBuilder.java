
package org.globalart.astrum.repository.content.fluent;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.hash.HashCode;

/**
 * Fluent API to create/find an asset; at this point we already know the asset path.
 *
 * @since 3.21
 */
public interface FluentAssetBuilder
{
  /**
   * Continue building the asset using the given kind.
   *
   * @since 3.24
   */
  FluentAssetBuilder kind(String kind);

  /**
   * Continue building the asset using the given owning component.
   */
  FluentAssetBuilder component(Component component);

  /**
   * Continue building this asset by converting a temporary blob into a permanent blob and attaching it to this asset.
   *
   * @since 3.30
   */
  FluentAssetBuilder blob(TempBlob blob);

  /**
   * Continue building this asset by attaching an existing blob to this asset.
   *
   * @since 3.30
   */
  FluentAssetBuilder blob(Blob blob, Map<HashAlgorithm, HashCode> checksums);

  /**
   * Continue building the asset using the given attributes.
   *
   * @since 3.31
   */
  FluentAssetBuilder attributes(String key, Object value);

  /**
   * Save the asset using details built so far. If the asset doesn't exist it is created otherwise the blob reference
   * is updated.
   *
   * @since 3.30
   */
  FluentAsset save();

  /**
   * Find by asset path if an asset exists using the details built so far.
   * Fields such as attributes, kind will be ignored.
   */
  Optional<FluentAsset> find();
}
