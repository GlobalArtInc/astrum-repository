package org.globalart.astrum.coreui;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_AUDIT_ENABLED_NAMED_VALUE;

/**
 * State contributor for Preview UI feature flags.
 *
 * Exposes the following state value to the frontend:
 * - previewAuditEnabled: Whether Preview UI audit DB persistence and audit log page are enabled
 *
 * previewAuditEnabled is system-property only (no KV store entry).
 */
@Component
public class PreviewUiStateContributor
    implements StateContributor
{
  private final boolean auditEnabled;

  @Autowired
  public PreviewUiStateContributor(@Value(PREVIEW_UI_AUDIT_ENABLED_NAMED_VALUE) final boolean auditEnabled) {
    this.auditEnabled = auditEnabled;
  }

  @Override
  @Nullable
  public Map<String, Object> getState() {
    return ImmutableMap.of("previewAuditEnabled", auditEnabled);
  }
}
