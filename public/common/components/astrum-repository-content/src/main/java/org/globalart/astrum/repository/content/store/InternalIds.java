
package org.globalart.astrum.repository.content.store;

import java.util.Optional;
import java.util.OptionalInt;

import org.globalart.astrum.common.entity.EntityHelper;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.RepositoryContent;
import org.globalart.astrum.repository.content.facet.ContentFacet;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkState;
import static java.lang.Integer.parseUnsignedInt;
import static java.lang.Integer.toHexString;
import static java.util.Optional.ofNullable;
import static java.util.OptionalInt.empty;

/**
 * Helper methods to access internal identifiers - USE WITH CAUTION!
 *
 * Where possible prefer the external identifiers such as component coordinates and asset paths.
 *
 * @since 3.24
 */
public class InternalIds
{
  private InternalIds() {
    // static utility class
  }

  // use Knuth's (reversible) multiplicative hash to scatter internal ids
  private static final long TO_EXTERNAL = 1327217885L; // 2 ^ 31 / golden ratio

  private static final long TO_INTERNAL = 828308341L; // multiplicative inverse

  public static EntityId toExternalId(final int internalId) {
    checkArgument(internalId > 0);
    return EntityHelper.id(toHexString((int) (internalId * TO_EXTERNAL))); // NOSONAR: we want truncation
  }

  /**
   * @throws NumberFormatException on invalid ids
   */
  public static int toInternalId(final EntityId externalId) {
    try {
      return (int) (parseUnsignedInt(externalId.getValue(), 16) * TO_INTERNAL); // NOSONAR: we want truncation
    }
    catch (NumberFormatException e) {
      // likely a big internal id that was passed as an external id, attempt to parse it directly
      return Integer.parseInt(externalId.getValue());
    }
  }

  public static Optional<Integer> contentRepositoryId(final Repository repository) {
    return repository.optionalFacet(ContentFacet.class).map(ContentFacet::contentRepositoryId);
  }

  public static int contentRepositoryId(final RepositoryContent content) {
    return checkInternalId(((AbstractRepositoryContent) unwrap(content)).repositoryId);
  }

  public static int contentRepositoryId(final ContentStoreEvent event) {
    return event.contentRepositoryId;
  }

  public static int internalComponentId(final Component component) {
    return checkInternalId(((ComponentData) unwrap(component)).componentId);
  }

  public static int internalAssetBlobId(final AssetBlob assetBlob) {
    return checkInternalId(((AssetBlobData) assetBlob).assetBlobId);
  }

  public static int internalAssetId(final Asset asset) {
    return checkInternalId(((AssetData) unwrap(asset)).assetId);
  }

  public static OptionalInt internalComponentId(final Asset asset) {
    AssetData assetData = (AssetData) unwrap(asset);
    return ofNullable(assetData.componentId).map(OptionalInt::of)
        .orElseGet(() -> assetData.component()
            .map(InternalIds::internalComponentId)
            .map(OptionalInt::of)
            .orElse(empty()));
  }

  public static OptionalInt internalAssetBlobId(final Asset asset) {
    return ofNullable(((AssetData) unwrap(asset)).assetBlobId).map(OptionalInt::of).orElse(empty());
  }

  private static int checkInternalId(final Integer internalId) {
    checkState(internalId != null, "Entity does not have an internal id; is it detached?");
    return internalId;
  }

  /**
   * Package-private so store classes in this package can reach the backing data object of a fluent wrapper without
   * duplicating the unwrapping logic.
   */
  static RepositoryContent unwrap(final RepositoryContent content) {
    return content instanceof WrappedContent<?> ? ((WrappedContent<?>) content).unwrap() : content;
  }
}
