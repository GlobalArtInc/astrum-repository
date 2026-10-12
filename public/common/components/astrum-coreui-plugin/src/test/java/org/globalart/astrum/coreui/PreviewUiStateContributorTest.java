package org.globalart.astrum.coreui;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class PreviewUiStateContributorTest
{
  @Test
  public void exposesPreviewAuditEnabledTrue() {
    PreviewUiStateContributor underTest = new PreviewUiStateContributor(true);

    assertThat(underTest.getState().get("previewAuditEnabled"), is(true));
  }

  @Test
  public void exposesPreviewAuditEnabledFalse() {
    PreviewUiStateContributor underTest = new PreviewUiStateContributor(false);

    assertThat(underTest.getState().get("previewAuditEnabled"), is(false));
  }

  @Test
  public void exposesNoFlagOfTheRetiredClassicUi() {
    assertThat(new PreviewUiStateContributor(false).getState().keySet().size(), is(1));
  }
}
