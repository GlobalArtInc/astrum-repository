
package org.globalart.astrum.repository.content.tasks.normalize.internal;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.tasks.normalize.NormalizationPriorityService;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * OSS implementation which does no prioritization of formats for normalization
 */
@Component
public class DefaultNormalizationPriorityService
    implements NormalizationPriorityService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, FormatStoreManager> prioritizedFormats;

  @Autowired
  public DefaultNormalizationPriorityService(final List<FormatStoreManager> formatStoreManagerList) {
    prioritizedFormats = QualifierUtil.buildQualifierBeanMap(formatStoreManagerList);
  }

  @Override
  public Map<String, FormatStoreManager> getPrioritizedFormats() {
    return prioritizedFormats;
  }
}
