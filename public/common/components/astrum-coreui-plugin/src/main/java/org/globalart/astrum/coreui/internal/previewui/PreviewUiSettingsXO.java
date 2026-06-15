
package org.globalart.astrum.coreui.internal.previewui;

/**
 * Transfer object for Preview UI settings.
 */
public class PreviewUiSettingsXO
{
  private boolean anonymousEnabled;

  private boolean loggedInEnabled;

  private boolean defaultToPreviewUi;

  private boolean disableLegacyUi;

  private boolean disableSwitchFeedback;

  public boolean isAnonymousEnabled() {
    return anonymousEnabled;
  }

  public void setAnonymousEnabled(final boolean anonymousEnabled) {
    this.anonymousEnabled = anonymousEnabled;
  }

  public boolean isLoggedInEnabled() {
    return loggedInEnabled;
  }

  public void setLoggedInEnabled(final boolean loggedInEnabled) {
    this.loggedInEnabled = loggedInEnabled;
  }

  public boolean isDefaultToPreviewUi() {
    return defaultToPreviewUi;
  }

  public void setDefaultToPreviewUi(final boolean defaultToPreviewUi) {
    this.defaultToPreviewUi = defaultToPreviewUi;
  }

  public boolean isDisableLegacyUi() {
    return disableLegacyUi;
  }

  public void setDisableLegacyUi(final boolean disableLegacyUi) {
    this.disableLegacyUi = disableLegacyUi;
  }

  public boolean isDisableSwitchFeedback() {
    return disableSwitchFeedback;
  }

  public void setDisableSwitchFeedback(final boolean disableSwitchFeedback) {
    this.disableSwitchFeedback = disableSwitchFeedback;
  }
}
