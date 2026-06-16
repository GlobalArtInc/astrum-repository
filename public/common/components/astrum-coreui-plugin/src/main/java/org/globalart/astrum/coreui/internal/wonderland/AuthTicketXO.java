
package org.globalart.astrum.coreui.internal.wonderland;

import jakarta.xml.bind.annotation.XmlAccessType;
import jakarta.xml.bind.annotation.XmlAccessorType;
import jakarta.xml.bind.annotation.XmlElement;
import jakarta.xml.bind.annotation.XmlRootElement;
import jakarta.xml.bind.annotation.XmlType;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Auth-ticket exchange object.
 *
 * @since 3.0
 */
@XmlAccessorType(XmlAccessType.FIELD)
@XmlType(name = "authTicket", propOrder = {
    "t"
})
@XmlRootElement(name = "authTicket")
public class AuthTicketXO
{
  @XmlElement(required = true)
  @JsonProperty("t")
  private String t;

  public String getT() {
    return t;
  }

  public void setT(String value) {
    this.t = value;
  }

  public AuthTicketXO withT(String value) {
    setT(value);
    return this;
  }
}
