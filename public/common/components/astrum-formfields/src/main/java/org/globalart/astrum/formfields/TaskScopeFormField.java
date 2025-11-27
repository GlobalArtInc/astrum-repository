
package org.globalart.astrum.formfields;

/**
 * Task scope field.
 */
public class TaskScopeFormField
    extends AbstractFormField<String>
{
  public TaskScopeFormField(final String id, boolean required, String initialValue) {
    super(id, "", "", required, null, initialValue);
  }

  @Override
  public String getType() {
    return "taskScope";
  }
}
