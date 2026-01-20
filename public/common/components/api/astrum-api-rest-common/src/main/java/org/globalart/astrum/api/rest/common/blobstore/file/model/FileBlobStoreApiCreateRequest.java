
package org.globalart.astrum.api.rest.common.blobstore.file.model;

import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.validation.constraint.NamePatternConstants;

/**
 * @since 3.19
 */
public class FileBlobStoreApiCreateRequest
    extends FileBlobStoreApiModel
{
  @NotEmpty(message = "Name is required")
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  private String name;

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  @Override
  public BlobStoreConfiguration toBlobStoreConfiguration(final BlobStoreConfiguration oldConfig) {
    BlobStoreConfiguration newConfig = super.toBlobStoreConfiguration(oldConfig);
    newConfig.setName(name);
    return newConfig;
  }
}
