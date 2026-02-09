
package org.globalart.astrum.repository.content.tasks.normalize;

import java.util.Map;

import org.globalart.astrum.repository.content.store.FormatStoreManager;

public interface NormalizationPriorityService
{
  Map<String, FormatStoreManager> getPrioritizedFormats();
}
