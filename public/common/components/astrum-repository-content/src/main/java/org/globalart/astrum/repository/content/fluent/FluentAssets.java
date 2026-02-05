
package org.globalart.astrum.repository.content.fluent;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.content.Asset;

/**
 * Fluent API for assets.
 *
 * @since 3.21
 */
public interface FluentAssets
    extends FluentQuery<FluentAsset>
{
  /**
   * Start building an asset, beginning with its path.
   */
  FluentAssetBuilder path(String path);

  /**
   * Interact with an existing asset.
   */
  FluentAsset with(Asset asset);

  /**
   * To include only group member content, ignoring assets in the group repository itself. By default, only group
   * repository content is returned
   */
  FluentQuery<FluentAsset> withOnlyGroupMemberContent();

  /**
   * To include both group content and group member content in the query. By default, only group
   * repository content is returned
   */
  FluentQuery<FluentAsset> withGroupMemberContent();

  /**
   * Query assets that have the given kind.
   *
   * @since 3.26
   */
  FluentQuery<FluentAsset> byKind(String kind);

  /**
   * Query assets that match the given filter.
   * <p>
   * A filter parameter of {@code foo} should be referred to in the filter string as <code>#{filterParams.foo}</code>
   * <p>
   * <b>WARNING</b> the filter string is appended to the query and should only contain trusted content!
   *
   * @since 3.26
   */
  FluentQuery<FluentAsset> byFilter(String filter, Map<String, Object> filterParams);

  /**
   * Find if an asset exists that has the given external id.
   *
   * @since 3.26
   */
  Optional<FluentAsset> find(EntityId externalId);
}
