
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.util.Map;

import org.globalart.astrum.common.entity.ContinuationAware;
import org.globalart.astrum.repository.content.AssetInfo;

/**
 * {@link AssetInfo} data backed by the content data store.
 *
 * @since 3.41
 */
public class AssetInfoData
    extends AbstractRepositoryContent
    implements AssetInfo, ContinuationAware
{
  private Integer assetId; // NOSONAR: internal id

  private Integer componentId;

  private String path;

  private String contentType;

  private String createdBy;

  private String createdByIp;

  private Map<String, String> checksums;

  private OffsetDateTime blobCreated;

  private OffsetDateTime lastDownloaded;

  private Long blobSize;

  private OffsetDateTime addedToRepository;

  @Override
  public String nextContinuationToken() {
    return Integer.toString(assetId);
  }

  @Override
  public Integer assetId() {
    return assetId;
  }

  @Override
  public Integer componentId() {
    return componentId;
  }

  @Override
  public String path() {
    return path;
  }

  @Override
  public String contentType() {
    return contentType;
  }

  @Override
  public String createdBy() {
    return createdBy;
  }

  @Override
  public String createdByIp() {
    return createdByIp;
  }

  @Override
  public OffsetDateTime lastDownloaded() {
    return lastDownloaded;
  }

  @Override
  public Long blobSize() {
    return blobSize;
  }

  @Override
  public Map<String, String> checksums() {
    return checksums;
  }

  @Override
  public OffsetDateTime blobCreated() {
    return blobCreated;
  }

  @Override
  public OffsetDateTime addedToRepository() {
    return addedToRepository;
  }

  @Override
  public String toString() {
    return "AssetInfoData{" +
        "assetId=" + assetId +
        ", componentId=" + componentId +
        ", path='" + path + '\'' +
        ", contentType='" + contentType + '\'' +
        ", createdBy='" + createdBy + '\'' +
        ", createdByIp='" + createdByIp + '\'' +
        ", checksums=" + checksums +
        ", blobCreated=" + blobCreated +
        ", lastDownloaded=" + lastDownloaded +
        ", blobSize=" + blobSize +
        ", addedToRepository=" + addedToRepository +
        '}';
  }
}
