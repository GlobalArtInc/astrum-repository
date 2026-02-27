
package org.globalart.astrum.repository.upload;

import java.util.Map;
import java.util.Optional;

public interface WithUploadField
{
  Map<String, String> getFields();

  default String getField(final String fieldName) {
    return Optional.ofNullable(getFields()).map(fields -> fields.get(fieldName)).orElse(null);
  }
}
