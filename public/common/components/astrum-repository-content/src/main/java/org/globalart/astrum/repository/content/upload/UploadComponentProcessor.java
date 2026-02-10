
package org.globalart.astrum.repository.content.upload;

import java.util.Optional;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.upload.UploadProcessor;
import org.globalart.astrum.repository.view.Content;

/**
 * Content implementation for {@link UploadProcessor}
 *
 * @since 3.29
 */
@org.springframework.stereotype.Component
public class UploadComponentProcessor
    implements UploadProcessor
{
  /**
   * Extract {@link EntityId} of {@link Component} from {@link Content}
   *
   * @param content uploaded {@link Content} with {@link Asset} in attributes
   * @return {@link EntityId} of {@link Component}
   */
  @Override
  public Optional<EntityId> extractId(final Content content) {
    Optional<Asset> asset = Optional.ofNullable(content.getAttributes().get(Asset.class));
    return asset.flatMap(Asset::component)
        .map(component -> InternalIds.toExternalId(InternalIds.internalComponentId(component)));
  }
}
