package org.globalart.astrum.coreui;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.kv.KeyValueStore;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_AUDIT_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_KEY;
import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_NAMED_VALUE;

/**
 * State contributor for Preview UI feature flags.
 *
 * Exposes the following state values to the frontend:
 * - anonymousEnabled: always true, anonymous users use Astrum One
 * - loggedInEnabled: always true, signed-in users use Astrum One
 * - defaultToPreviewUi: always true, every dashboard visit opens Astrum One
 * - disableLegacyUi: always true, the Classic UI is retired and its URLs redirect to Astrum One
 * - disableSwitchFeedback: Whether to skip the feedback prompt on Classic UI switch
 * - previewAuditEnabled: Whether Preview UI audit DB persistence and audit log page are enabled
 *
 * The four interface flags are fixed: values saved in the KeyValueStore or set in astrum.properties
 * by earlier releases are ignored, so the Classic UI cannot be switched back on.
 * disableSwitchFeedback is read from the KeyValueStore with a system property fallback.
 * previewAuditEnabled is system-property only (no KV store entry).
 */
@Component
public class PreviewUiStateContributor
    implements StateContributor
{
  private final KeyValueStore keyValueStore;

  private final boolean defaultSwitchFeedbackDisabled;

  private final boolean auditEnabled;

  @Autowired
  public PreviewUiStateContributor(
      final KeyValueStore keyValueStore,
      @Value(PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_NAMED_VALUE) final boolean defaultSwitchFeedbackDisabled,
      @Value(PREVIEW_UI_AUDIT_ENABLED_NAMED_VALUE) final boolean auditEnabled)
  {
    this.keyValueStore = checkNotNull(keyValueStore);
    this.defaultSwitchFeedbackDisabled = defaultSwitchFeedbackDisabled;
    this.auditEnabled = auditEnabled;
  }

  @Override
  @Nullable
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        "anonymousEnabled", true,
        "loggedInEnabled", true,
        "defaultToPreviewUi", true,
        "disableLegacyUi", true,
        "disableSwitchFeedback",
        keyValueStore.getBoolean(PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_KEY).orElse(defaultSwitchFeedbackDisabled),
        "previewAuditEnabled", auditEnabled);
  }
}
