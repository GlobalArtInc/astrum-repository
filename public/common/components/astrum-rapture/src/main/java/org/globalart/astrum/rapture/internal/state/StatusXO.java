
package org.globalart.astrum.rapture.internal.state;

/**
 * Status exchange object.
 *
 * @since 3.0
 */
public class StatusXO
{
  private String edition;

  private String version;

  private String buildRevision;

  private String buildTimestamp;

  public StatusXO() {
  }

  public StatusXO(String edition, String version, String buildRevision, String buildTimestamp) {
    this.edition = edition;
    this.version = version;
    this.buildRevision = buildRevision;
    this.buildTimestamp = buildTimestamp;
  }

  public String getEdition() {
    return edition;
  }

  public void setEdition(String edition) {
    this.edition = edition;
  }

  public String getVersion() {
    return version;
  }

  public void setVersion(String version) {
    this.version = version;
  }

  public String getBuildRevision() {
    return buildRevision;
  }

  public void setBuildRevision(String buildRevision) {
    this.buildRevision = buildRevision;
  }

  public String getBuildTimestamp() {
    return buildTimestamp;
  }

  public void setBuildTimestamp(String buildTimestamp) {
    this.buildTimestamp = buildTimestamp;
  }

  @Override
  public String toString() {
    return "StatusXO{" +
        "edition='" + edition + '\'' +
        ", version='" + version + '\'' +
        ", buildRevision='" + buildRevision + '\'' +
        ", buildTimestamp='" + buildTimestamp + '\'' +
        '}';
  }
}
