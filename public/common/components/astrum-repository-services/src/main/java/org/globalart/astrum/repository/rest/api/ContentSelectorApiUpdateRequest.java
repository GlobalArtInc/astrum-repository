
package org.globalart.astrum.repository.rest.api;

import java.util.Objects;
import jakarta.validation.constraints.NotBlank;

import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.EXPRESSION_DESCRIPTION;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.EXPRESSION_EXAMPLE;

/**
 * ContentSelector transfer object for REST APIs.
 */
@Schema
public class ContentSelectorApiUpdateRequest
    implements ValidatableContentSelectorRequest
{
  @Schema(description = "An optional description of this content selector")
  private String description;

  @Schema(description = EXPRESSION_DESCRIPTION, example = EXPRESSION_EXAMPLE)
  @NotBlank
  private String expression;

  public String getDescription() {
    return description;
  }

  public void setDescription(String description) {
    this.description = description;
  }

  public String getExpression() {
    return expression;
  }

  public void setExpression(String expression) {
    this.expression = expression;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ContentSelectorApiUpdateRequest that = (ContentSelectorApiUpdateRequest) o;
    return Objects.equals(description, that.description) &&
        Objects.equals(expression, that.expression);
  }

  @Override
  public int hashCode() {
    return Objects.hash(description, expression);
  }
}
