
package org.globalart.astrum.cleanup.preview;

import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_PREVIEW_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_PREVIEW_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_RETAIN_ALL_FORMATS;
import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_RETAIN_ALL_FORMATS_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class CleanupPreviewStateContributor
    implements StateContributor
{
  private final boolean cleanupPreviewEnabled;

  private final boolean cleanupRetainAllFormatsEnabled;

  @Autowired
  public CleanupPreviewStateContributor(
      @Value(CLEANUP_PREVIEW_ENABLED_NAMED_VALUE) final boolean cleanupPreviewEnabled,
      @Value(CLEANUP_RETAIN_ALL_FORMATS_NAMED_VALUE) final boolean cleanupRetainAllFormatsEnabled)
  {
    this.cleanupPreviewEnabled = cleanupPreviewEnabled;
    this.cleanupRetainAllFormatsEnabled = cleanupRetainAllFormatsEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        CLEANUP_PREVIEW_ENABLED, cleanupPreviewEnabled,
        CLEANUP_RETAIN_ALL_FORMATS, cleanupRetainAllFormatsEnabled);
  }
}
