
package org.globalart.astrum.repository.upload;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.repository.view.PartPayload;

/**
 * An asset being uploaded.
 *
 * @since 3.7
 */
public class AssetUpload
    implements WithUploadField
{
  private Map<String, String> fields = new HashMap<>();

  private PartPayload payload;

  @Override
  public Map<String, String> getFields() {
    return fields;
  }

  public PartPayload getPayload() {
    return payload;
  }

  public void setFields(final Map<String, String> fields) {
    this.fields = fields;
  }

  public void setPayload(final PartPayload payload) {
    this.payload = payload;
  }
}
