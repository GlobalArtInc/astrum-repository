
package org.globalart.astrum.repository.rest.api;

import java.util.Map;
import java.util.Objects;

import org.globalart.astrum.repository.upload.UploadFieldDefinition;

/**
 * Upload field definition transfer object for rest api
 */
public class UploadFieldDefinitionXO
{
  private String name;

  private String type;

  private String description;

  private boolean optional;

  private String group;

  private String groupHelpText;

  private Map<String, String> options;

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  public String getDescription() {
    return description;
  }

  public void setDescription(String description) {
    this.description = description;
  }

  public boolean isOptional() {
    return optional;
  }

  public void setOptional(boolean optional) {
    this.optional = optional;
  }

  public String getGroup() {
    return group;
  }

  public void setGroup(String group) {
    this.group = group;
  }

  public String getGroupHelpText() {
    return groupHelpText;
  }

  public void setGroupHelpText(String groupHelpText) {
    this.groupHelpText = groupHelpText;
  }

  public Map<String, String> getOptions() {
    return options;
  }

  public void setOptions(Map<String, String> options) {
    this.options = options;
  }

  public static UploadFieldDefinitionXO from(final UploadFieldDefinition uploadFieldDefinition) {
    return builder()
        .name(uploadFieldDefinition.getName())
        .type(uploadFieldDefinition.getType().name())
        .description(uploadFieldDefinition.getHelpText())
        .optional(uploadFieldDefinition.isOptional())
        .group(uploadFieldDefinition.getGroup())
        .groupHelpText(uploadFieldDefinition.getGroupHelpText())
        .options(uploadFieldDefinition.getOptions())
        .build();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    UploadFieldDefinitionXO that = (UploadFieldDefinitionXO) o;
    return Objects.equals(name, that.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name);
  }

  @Override
  public String toString() {
    return "UploadFieldDefinitionXO{" +
        "name='" + name + '\'' +
        ", type='" + type + '\'' +
        ", description='" + description + '\'' +
        ", optional=" + optional +
        ", group='" + group + '\'' +
        ", groupHelpText='" + groupHelpText + '\'' +
        ", options=" + options +
        '}';
  }

  public static UploadFieldDefinitionXOBuilder builder() {
    return new UploadFieldDefinitionXOBuilder();
  }

  public static class UploadFieldDefinitionXOBuilder
  {
    private String name;

    private String type;

    private String description;

    private boolean optional;

    private String group;

    private String groupHelpText;

    private Map<String, String> options;

    public UploadFieldDefinitionXOBuilder name(String name) {
      this.name = name;
      return this;
    }

    public UploadFieldDefinitionXOBuilder type(String type) {
      this.type = type;
      return this;
    }

    public UploadFieldDefinitionXOBuilder description(String description) {
      this.description = description;
      return this;
    }

    public UploadFieldDefinitionXOBuilder optional(boolean optional) {
      this.optional = optional;
      return this;
    }

    public UploadFieldDefinitionXOBuilder group(String group) {
      this.group = group;
      return this;
    }

    public UploadFieldDefinitionXOBuilder groupHelpText(String groupHelpText) {
      this.groupHelpText = groupHelpText;
      return this;
    }

    public UploadFieldDefinitionXOBuilder options(Map<String, String> options) {
      this.options = options;
      return this;
    }

    public UploadFieldDefinitionXO build() {
      UploadFieldDefinitionXO uploadFieldDefinitionXO = new UploadFieldDefinitionXO();
      uploadFieldDefinitionXO.setName(name);
      uploadFieldDefinitionXO.setType(type);
      uploadFieldDefinitionXO.setDescription(description);
      uploadFieldDefinitionXO.setOptional(optional);
      uploadFieldDefinitionXO.setGroup(group);
      uploadFieldDefinitionXO.setGroupHelpText(groupHelpText);
      uploadFieldDefinitionXO.setOptions(options);
      return uploadFieldDefinitionXO;
    }
  }
}
