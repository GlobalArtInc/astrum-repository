
package org.globalart.astrum.rest;

import jakarta.xml.bind.annotation.XmlRootElement;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Fault exchange object.
 *
 * @since 3.0
 */
@XmlRootElement(name = "fault")
public class FaultXO
{
  @JsonProperty
  private String id;

  @JsonProperty
  private String message;

  public FaultXO() {
    // empty
  }

  public FaultXO(final String id, final String message) {
    this.id = id;
    this.message = message;
  }

  public FaultXO(final String id, final Throwable cause) {
    this(id, cause.toString());
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getMessage() {
    return message;
  }

  public void setMessage(final String message) {
    this.message = message;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        ", message='" + message + '\'' +
        '}';
  }
}
