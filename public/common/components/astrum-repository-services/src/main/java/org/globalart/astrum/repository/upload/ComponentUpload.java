
package org.globalart.astrum.repository.upload;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * A component being uploaded.
 *
 * @since 3.7
 */
public class ComponentUpload
    implements WithUploadField
{
  private List<AssetUpload> assetUploads = new ArrayList<>();

  private Map<String, String> fields = new HashMap<>();

  public List<AssetUpload> getAssetUploads() {
    return assetUploads;
  }

  @Override
  public Map<String, String> getFields() {
    return fields;
  }

  public void setAssetUploads(final List<AssetUpload> assetUploads) {
    this.assetUploads = assetUploads;
  }

  public void setFields(final Map<String, String> fields) {
    this.fields = fields;
  }
}
