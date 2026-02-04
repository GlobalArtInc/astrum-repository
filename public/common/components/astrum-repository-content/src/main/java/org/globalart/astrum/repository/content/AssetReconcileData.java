
package org.globalart.astrum.repository.content;

import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.common.entity.ContinuationAware;

public class AssetReconcileData
    implements ContinuationAware
{
  private BlobRef blobRef;

  private String repository;

  private String path;

  public Integer getAssetBlobId() {
    return assetBlobId;
  }

  private Integer assetBlobId;

  public BlobRef getBlobRef() {
    return blobRef;
  }

  public void setBlobRef(final BlobRef blobRef) {
    this.blobRef = blobRef;
  }

  public String getRepository() {
    return repository;
  }

  public void setRepository(final String repository) {
    this.repository = repository;
  }

  public String getPath() {
    return path;
  }

  public void setPath(final String path) {
    this.path = path;
  }

  @Override
  public String nextContinuationToken() {
    return Integer.toString(assetBlobId);
  }

  public void setAssetBlobId(final Integer assetBlobId) {
    this.assetBlobId = assetBlobId;
  }
}
