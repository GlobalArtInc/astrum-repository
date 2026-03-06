
package org.globalart.astrum.cleanup.internal.rest;

public class CleanupPolicyRepositoryXO
{
  private String name;

  private String format;

  private String type;

  public CleanupPolicyRepositoryXO() {
  }

  public CleanupPolicyRepositoryXO(final String name, final String format, final String type) {
    this.name = name;
    this.format = format;
    this.type = type;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getFormat() {
    return format;
  }

  public void setFormat(final String format) {
    this.format = format;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }
}
