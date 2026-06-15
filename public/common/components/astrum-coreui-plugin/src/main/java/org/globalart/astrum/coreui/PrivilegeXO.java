
package org.globalart.astrum.coreui;

import java.util.Map;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.security.privilege.UniquePrivilegeId;
import org.globalart.astrum.security.privilege.UniquePrivilegeName;
import org.globalart.astrum.validation.constraint.NamePatternConstants;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

/**
 * Privilege exchange object.
 */
public class PrivilegeXO
{
  @NotBlank(groups = Update.class)
  @UniquePrivilegeId(groups = Create.class)
  private String id;

  @NotBlank(groups = Update.class)
  private String version;

  @NotBlank
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @UniquePrivilegeName(groups = Create.class)
  private String name;

  private String description;

  @NotBlank
  private String type;

  private Boolean readOnly;

  @NotEmpty
  private Map<String, String> properties;

  private String permission;

  public String getId() {
    return id;
  }

  public PrivilegeXO withId(final String id) {
    this.id = id;
    return this;
  }

  public String getVersion() {
    return version;
  }

  public PrivilegeXO withVersion(final String version) {
    this.version = version;
    return this;
  }

  public String getName() {
    return name;
  }

  public PrivilegeXO withName(final String name) {
    this.name = name;
    return this;
  }

  public String getDescription() {
    return description;
  }

  public PrivilegeXO withDescription(final String description) {
    this.description = description;
    return this;
  }

  public String getType() {
    return type;
  }

  public PrivilegeXO withType(final String type) {
    this.type = type;
    return this;
  }

  public Boolean getReadOnly() {
    return readOnly;
  }

  public PrivilegeXO withReadOnly(final Boolean readOnly) {
    this.readOnly = readOnly;
    return this;
  }

  public Map<String, String> getProperties() {
    return properties;
  }

  public PrivilegeXO withProperties(final Map<String, String> properties) {
    this.properties = properties;
    return this;
  }

  public String getPermission() {
    return permission;
  }

  public PrivilegeXO withPermission(final String permission) {
    this.permission = permission;
    return this;
  }

  @Override
  public String toString() {
    return "PrivilegeXO{" +
        "id='" + id + '\'' +
        ", version='" + version + '\'' +
        ", name='" + name + '\'' +
        ", description='" + description + '\'' +
        ", type='" + type + '\'' +
        ", readOnly=" + readOnly +
        ", properties=" + properties +
        ", permission='" + permission + '\'' +
        '}';
  }
}
