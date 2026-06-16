
package org.globalart.astrum.coreui.internal.wonderland;

import jakarta.xml.bind.annotation.XmlAccessType;
import jakarta.xml.bind.annotation.XmlAccessorType;
import jakarta.xml.bind.annotation.XmlElement;
import jakarta.xml.bind.annotation.XmlRootElement;
import jakarta.xml.bind.annotation.XmlType;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Status exchange object.
 *
 * @since 3.0
 */
@XmlAccessorType(XmlAccessType.FIELD)
@XmlType(name = "status", propOrder = {
    "edition",
    "version",
    "isCloud"
})
@XmlRootElement(name = "status")
public class StatusXO
{
  @XmlElement(required = true)
  @JsonProperty("edition")
  private String edition;

  @XmlElement(required = true)
  @JsonProperty("version")
  private String version;

  @XmlElement(required = true)
  @JsonProperty("isCloud")
  private boolean isCloud;

  public String getEdition() {
    return edition;
  }

  public void setEdition(final String edition) {
    this.edition = edition;
  }

  public String getVersion() {
    return version;
  }

  public void setVersion(final String version) {
    this.version = version;
  }

  public boolean getIsCloud() {
    return isCloud;
  }

  public void setIsCloud(final boolean isCloud) {
    this.isCloud = isCloud;
  }
}
