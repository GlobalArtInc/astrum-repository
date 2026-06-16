
package org.globalart.astrum.coreui.internal.wonderland;

import jakarta.xml.bind.annotation.XmlAccessType;
import jakarta.xml.bind.annotation.XmlAccessorType;
import jakarta.xml.bind.annotation.XmlElement;
import jakarta.xml.bind.annotation.XmlRootElement;
import jakarta.xml.bind.annotation.XmlType;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Property exchange object.
 *
 * @since 3.0
 */
@XmlAccessorType(XmlAccessType.FIELD)
@XmlType(name = "property", propOrder = {
    "key",
    "value"
})
@XmlRootElement(name = "property")
public class PropertyXO
{
  @XmlElement(required = true)
  @JsonProperty("key")
  private String key;

  @JsonProperty("value")
  private String value;

  public String getKey() {
    return key;
  }

  public void setKey(String value) {
    this.key = value;
  }

  public String getValue() {
    return value;
  }

  public void setValue(String value) {
    this.value = value;
  }

  public PropertyXO withKey(String value) {
    setKey(value);
    return this;
  }

  public PropertyXO withValue(String value) {
    setValue(value);
    return this;
  }
}
