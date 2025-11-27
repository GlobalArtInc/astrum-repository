
package org.globalart.astrum.formfields;

import java.util.Map;

/**
 * The model for a combo field allowing for selection of Repository Targets.
 *
 * @since 2.5
 */
public class RepoTargetComboFormField
    extends Combobox<String>
{

  public static final String DEFAULT_HELP_TEXT = "Select the repository target to apply ";

  public static final String DEFAULT_LABEL = "Repository Target";

  public RepoTargetComboFormField(
      String id,
      String label,
      String helpText,
      boolean required,
      String regexValidation)
  {
    super(id, label, helpText, required);
    withRegexValidation(regexValidation);
  }

  public RepoTargetComboFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public RepoTargetComboFormField(String id, boolean required) {
    super(id, DEFAULT_LABEL, DEFAULT_HELP_TEXT, required);
  }

  public RepoTargetComboFormField(String id) {
    super(id, DEFAULT_LABEL, DEFAULT_HELP_TEXT, false);
  }

  public String getType() {
    return "repo-target";
  }

  /**
   * @since 3.0
   */
  @Override
  public String getStoreApi() {
    return "coreui_RepositoryTarget.read";
  }

  /**
   * @since 3.0
   */
  @Override
  public Map<String, String> getStoreFilters() {
    return null;
  }
}
