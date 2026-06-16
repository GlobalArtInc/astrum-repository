
package org.globalart.astrum.coreui.internal.wonderland;

import jakarta.xml.bind.annotation.XmlAccessType;
import jakarta.xml.bind.annotation.XmlAccessorType;
import jakarta.xml.bind.annotation.XmlElement;
import jakarta.xml.bind.annotation.XmlRootElement;
import jakarta.xml.bind.annotation.XmlType;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Auth-token exchange object.
 *
 * @since 3.0
 */
@XmlAccessorType(XmlAccessType.FIELD)
@XmlType(name = "authToken", propOrder = {
    "u",
    "p"
})
@XmlRootElement(name = "authToken")
public class AuthTokenXO
{
  @XmlElement(required = true)
  @JsonProperty("u")
  private String u;

  @XmlElement(required = true)
  @JsonProperty("p")
  private String p;

  public String getU() {
    return u;
  }

  public void setU(String value) {
    this.u = value;
  }

  public String getP() {
    return p;
  }

  public void setP(String value) {
    this.p = value;
  }

  public AuthTokenXO withU(String value) {
    setU(value);
    return this;
  }

  public AuthTokenXO withP(String value) {
    setP(value);
    return this;
  }
}
