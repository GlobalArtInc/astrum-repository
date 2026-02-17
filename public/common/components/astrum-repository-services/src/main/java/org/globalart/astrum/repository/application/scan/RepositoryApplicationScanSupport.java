
package org.globalart.astrum.repository.application.scan;

import java.util.Collections;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;

public interface RepositoryApplicationScanSupport
{
  boolean isSupported();

  String getStageTypeId(Repository repository);

  static String getStageTypeIdFromAttributes(final Repository repository, final String... keys) {
    Configuration configuration = repository.getConfiguration();
    if (configuration == null) {
      return null;
    }
    Map<String, Map<String, Object>> attributes = configuration.getAttributes();
    if (attributes == null) {
      return null;
    }
    Object value = attributes;
    for (String key : keys) {
      if (value instanceof Map) {
        value = ((Map) value).getOrDefault(key, Collections.emptyMap());
      }
    }
    if ("RELEASE".equals(value)) {
      return "release";
    }
    return null;
  }
}
