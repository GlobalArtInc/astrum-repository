package org.globalart.astrum.repository.terraform;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.terraform.TerraformPaths.MODULES;

/**
 * Address of a module inside a repository: {@code namespace/name/system}, optionally at one version.
 */
public record ModuleCoordinates(String namespace, String name, String system, @Nullable String version)
{
  public ModuleCoordinates {
    checkArgument(TerraformPaths.isValidName(namespace), "Invalid module namespace: %s", namespace);
    checkArgument(TerraformPaths.isValidName(name), "Invalid module name: %s", name);
    checkArgument(TerraformPaths.isValidName(system), "Invalid module system: %s", system);
    checkArgument(version == null || TerraformPaths.isValidVersion(version), "Invalid module version: %s", version);
  }

  /**
   * The module address without host, as Terraform users write it.
   */
  public String address() {
    return namespace + "/" + name + "/" + system;
  }

  /**
   * The name of the module version component, whose namespace is the module namespace: {@code name/system}.
   */
  public String componentName() {
    return name + "/" + system;
  }

  public String basePath() {
    return MODULES + address();
  }

  public String versionsPath() {
    return basePath() + "/versions";
  }

  public String directory() {
    return basePath() + "/" + checkNotNull(version);
  }

  public String archiveFileName(final String extension) {
    return name + "-" + system + "-" + checkNotNull(version) + "." + extension;
  }

  public String archivePath(final String extension) {
    return directory() + "/" + archiveFileName(extension);
  }
}
