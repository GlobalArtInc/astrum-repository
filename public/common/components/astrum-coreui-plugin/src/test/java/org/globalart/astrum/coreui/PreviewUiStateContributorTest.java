
package org.globalart.astrum.coreui;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.kv.KeyValueStore;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_KEY;

@ExtendWith(MockitoExtension.class)
class PreviewUiStateContributorTest
{
  @Mock
  private KeyValueStore keyValueStore;

  @BeforeEach
  public void setUp() {
    when(keyValueStore.getBoolean(anyString())).thenReturn(Optional.empty());
  }

  @Test
  public void getStateIncludesDisableSwitchFeedback() {
    when(keyValueStore.getBoolean("preview.ui.anonymous.enabled")).thenReturn(Optional.of(false));
    when(keyValueStore.getBoolean("preview.ui.loggedin.enabled")).thenReturn(Optional.of(true));
    when(keyValueStore.getBoolean("preview.ui.default.enabled")).thenReturn(Optional.of(false));
    when(keyValueStore.getBoolean("preview.ui.legacy.disabled")).thenReturn(Optional.of(false));
    when(keyValueStore.getBoolean(PREVIEW_UI_SWITCH_FEEDBACK_DISABLED_KEY)).thenReturn(Optional.of(true));

    PreviewUiStateContributor underTest = contributor(false);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("disableSwitchFeedback"), is(true));
  }

  @Test
  public void exposesPreviewAuditEnabledTrue() {
    PreviewUiStateContributor underTest = contributor(true);

    assertThat(underTest.getState().get("previewAuditEnabled"), is(true));
  }

  @Test
  public void exposesPreviewAuditEnabledFalse() {
    PreviewUiStateContributor underTest = contributor(false);

    assertThat(underTest.getState().get("previewAuditEnabled"), is(false));
  }

  @Test
  public void previewAuditEnabledIsNotReadFromKeyValueStore() {
    // The audit flag is system-property only by design (ASTRUM-52060) — KV store lookups for it
    // should never happen, so an admin cannot accidentally enable audit persistence via the
    // preview-ui settings page. Stub is lenient because the production code never issues this
    // call — that absence is the invariant under test.
    lenient().when(keyValueStore.getBoolean("preview.ui.audit.enabled")).thenReturn(Optional.of(true));

    PreviewUiStateContributor underTest = contributor(false);

    assertThat(underTest.getState().get("previewAuditEnabled"), is(false));
  }

  private PreviewUiStateContributor contributor(final boolean auditEnabled) {
    return new PreviewUiStateContributor(keyValueStore, false, false, false, false, false, auditEnabled);
  }
}
