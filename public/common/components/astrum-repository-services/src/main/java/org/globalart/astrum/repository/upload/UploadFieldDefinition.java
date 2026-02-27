
package org.globalart.astrum.repository.upload;

import java.util.Map;
import java.util.Objects;

import org.globalart.astrum.common.text.Strings2;

/**
 * Description of a field used when uploading a component.
 *
 * @since 3.7
 */
public class UploadFieldDefinition
{
  public enum Type
  {
    BOOLEAN,
    STRING,
    SELECT
  }

  private String displayName;

  private String helpText;

  private String name;

  private boolean optional;

  private Type type;

  private String group;

  private String groupHelpText;

  private Map<String, String> options;

  public UploadFieldDefinition(final String name, final boolean optional, final Type type) {
    this(name, Strings2.capitalize(name), null, optional, type, null);
  }

  public UploadFieldDefinition(final String name, final boolean optional, final Type type, final String group) {
    this(name, Strings2.capitalize(name), null, optional, type, group);
  }

  public UploadFieldDefinition(final String name, final String helpText, final boolean optional, final Type type) {
    this(name, Strings2.capitalize(name), helpText, optional, type, null);
  }

  public UploadFieldDefinition(
      final String name,
      final String helpText,
      final boolean optional,
      final Type type,
      final String group)
  {
    this(name, Strings2.capitalize(name), helpText, optional, type, group);
  }

  public UploadFieldDefinition(
      final String name,
      final String displayName,
      final String helpText,
      final boolean optional,
      final Type type,
      final String group)
  {
    this(name, displayName, helpText, optional, type, group, null);
  }

  public UploadFieldDefinition(
      final String name,
      final String displayName,
      final String helpText,
      final boolean optional,
      final Type type,
      final String group,
      final Map<String, String> options)
  {
    this(name, displayName, helpText, optional, type, group, null, options);
  }

  public UploadFieldDefinition(
      final String name,
      final String displayName,
      final String helpText,
      final boolean optional,
      final Type type,
      final String group,
      final String groupHelpText,
      final Map<String, String> options)
  {
    this.name = name;
    this.displayName = displayName;
    this.helpText = helpText;
    this.optional = optional;
    this.type = type;
    this.group = group;
    this.groupHelpText = groupHelpText;
    this.options = options;
  }

  /**
   * The name of the field
   */
  public String getName() {
    return name;
  }

  /**
   * The name to be displayed in UI
   */
  public String getDisplayName() {
    return this.displayName;
  }

  /**
   * The help text to be displayed in UI
   */
  public String getHelpText() {
    return this.helpText;
  }

  /**
   * The type of the field
   */
  public Type getType() {
    return type;
  }

  /**
   * Indicates whether this field is required
   */
  public boolean isOptional() {
    return optional;
  }

  /**
   * The name of a group this field belongs to
   */
  public String getGroup() {
    return this.group;
  }

  /**
   * The help text for the group this field belongs to
   */
  public String getGroupHelpText() {
    return this.groupHelpText;
  }

  public void setGroupHelpText(final String groupHelpText) {
    this.groupHelpText = groupHelpText;
  }

  /**
   * The options available for SELECT type fields
   */
  public Map<String, String> getOptions() {
    return this.options;
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, displayName, helpText, type, group, groupHelpText, optional, options);
  }

  @Override
  public boolean equals(final Object obj) {
    if (this == obj) { // Early return to speed up the equals checks for the same object
      return true;
    }

    if (obj == null || getClass() != obj.getClass()) {
      return false;
    }

    UploadFieldDefinition other = (UploadFieldDefinition) obj;

    if (!Objects.equals(name, other.name)) {
      return false;
    }

    if (!Objects.equals(displayName, other.displayName)) {
      return false;
    }

    if (!Objects.equals(helpText, other.helpText)) {
      return false;
    }

    if (!Objects.equals(type, other.type)) {
      return false;
    }

    if (!Objects.equals(group, other.group)) {
      return false;
    }

    if (!Objects.equals(groupHelpText, other.groupHelpText)) {
      return false;
    }

    if (!Objects.equals(options, other.options)) {
      return false;
    }

    return optional == other.optional;
  }
}
