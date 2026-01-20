
package org.globalart.astrum.api.rest.common.blobstore.file.model;

import jakarta.validation.constraints.NotEmpty;

import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreApiModel;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.blobstore.file.internal.FileBlobStoreDescriptor;

import io.swagger.v3.oas.annotations.media.Schema;

public class FileBlobStoreApiModel
    extends BlobStoreApiModel
{
  @Schema(description = FileBlobStoreDescriptor.PATH_HELP_TEXT)
  @NotEmpty(message = "Path is required")
  private String path;

  public FileBlobStoreApiModel() {
    super();
  }

  public FileBlobStoreApiModel(final BlobStoreConfiguration configuration) {
    super(configuration);
    this.path = configuration.attributes(FileBlobStore.CONFIG_KEY).get(FileBlobStore.PATH_KEY, String.class);
  }

  public String getPath() {
    return path;
  }

  public void setPath(final String path) {
    this.path = path;
  }

  @Override
  public BlobStoreConfiguration toBlobStoreConfiguration(final BlobStoreConfiguration configuration) {
    BlobStoreConfiguration newConfig = super.toBlobStoreConfiguration(configuration);
    newConfig.setType(FileBlobStore.TYPE);
    newConfig.attributes(FileBlobStore.CONFIG_KEY).set(FileBlobStore.PATH_KEY, path);
    return newConfig;
  }
}
