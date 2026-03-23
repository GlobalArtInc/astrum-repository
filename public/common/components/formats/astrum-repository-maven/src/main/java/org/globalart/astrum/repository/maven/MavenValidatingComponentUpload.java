
package org.globalart.astrum.repository.maven;

import java.util.Optional;

import org.globalart.astrum.repository.upload.AssetUpload;
import org.globalart.astrum.repository.upload.ComponentUpload;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.ValidatingComponentUpload;

/**
 * A holder of {@link ComponentUpload} that's meant to validate it based on provided {@link UploadDefinition} for Maven
 *
 * @since 3.8
 */
public class MavenValidatingComponentUpload
    extends ValidatingComponentUpload
{
  private static final String EXTENSION = "extension";

  private static final String CLASSIFIER = "classifier";

  public MavenValidatingComponentUpload(
      final UploadDefinition uploadDefinition,
      final ComponentUpload componentUpload)
  {
    super(uploadDefinition, componentUpload);
  }

  @Override
  protected void validateRequiredComponentFieldPresent() {
    if (!findPomAsset().isPresent()) {
      super.validateRequiredComponentFieldPresent();
    }
  }

  private Optional<AssetUpload> findPomAsset() {
    return componentUpload.getAssetUploads()
        .stream()
        .filter(asset -> "pom".equals(asset.getField(EXTENSION)))
        .findFirst();
  }
}
