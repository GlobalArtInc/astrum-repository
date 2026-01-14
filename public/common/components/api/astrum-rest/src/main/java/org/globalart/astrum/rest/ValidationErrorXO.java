
package org.globalart.astrum.rest;

import jakarta.xml.bind.annotation.XmlRootElement;

import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Validation error exchange object.
 *
 * @since 3.0
 */
@XmlRootElement(name = "validationError")
public class ValidationErrorXO
{
  /**
   * Denotes that validation does not applies to a specific value.
   */
  public static final String GENERIC = "*";

  /**
   * Identifies the value value that is failing validation. A value of "*" denotes that validation
   * does not applies to a specific value.
   *
   * E.g. "name".
   */
  @JsonProperty
  private String id;

  /**
   * Description of failing validation.
   *
   * E.g. "Name cannot be null".
   */
  @JsonProperty
  private String message;

  public ValidationErrorXO() {
    this.id = GENERIC;
  }

  /**
   * Creates a validation error that does not applies to a specific value.
   *
   * @param message validation description
   */
  public ValidationErrorXO(final String message) {
    this(GENERIC, message);
  }

  /**
   * Creates a validation error for a specific value.
   *
   * @param id identifier of value failing validation.
   * @param message validation description
   */
  public ValidationErrorXO(final String id, final String message) {
    this.id = id == null ? GENERIC : id;
    this.message = message;
  }

  /**
   * @return identifier of value failing validation (never null). A value of "*" denotes that validation does
   *         not applies to a specific value.
   */
  public String getId() {
    return id;
  }

  /**
   * @param id of value failing validation
   */
  public void setId(final String id) {
    this.id = id == null ? GENERIC : id;
  }

  /**
   * @param id of value failing validation
   * @return itself, for fluent api usage
   */
  public ValidationErrorXO withId(final String id) {
    setId(id);
    return this;
  }

  /**
   * @return validation description
   */
  public String getMessage() {
    return message;
  }

  /**
   * @param message validation description
   */
  public void setMessage(final String message) {
    this.message = message;
  }

  /**
   * @param message validation description
   * @return itself, for fluent api usage
   */
  public ValidationErrorXO withMessage(final String message) {
    this.message = message;
    return this;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        ", message='" + message + '\'' +
        '}';
  }
}
