package org.globalart.astrum.repository.terraform;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Operating system and architecture of a provider package, {@code linux_amd64} in Terraform's notation.
 */
public record Platform(String os, String arch)
    implements Comparable<Platform>
{
  public Platform {
    checkArgument(TerraformPaths.isValidPlatformPart(os), "Invalid operating system: %s", os);
    checkArgument(TerraformPaths.isValidPlatformPart(arch), "Invalid architecture: %s", arch);
  }

  @Override
  public String toString() {
    return os + "_" + arch;
  }

  @Override
  public int compareTo(final Platform other) {
    return toString().compareTo(other.toString());
  }
}
