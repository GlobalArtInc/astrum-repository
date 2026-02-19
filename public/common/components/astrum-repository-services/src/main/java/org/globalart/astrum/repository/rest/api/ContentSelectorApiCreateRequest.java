
package org.globalart.astrum.repository.rest.api;

import java.util.Objects;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.validation.constraint.NamePatternConstants;

import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.DESCRIPTION_DESCRIPTION;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.EXPRESSION_DESCRIPTION;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.EXPRESSION_EXAMPLE;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.NAME_DESCRIPTION;

/**
 * ContentSelector transfer object for REST APIs.
 */
@Schema
public class ContentSelectorApiCreateRequest
    implements ValidatableContentSelectorRequest
{
  @Schema(description = NAME_DESCRIPTION)
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotBlank
  private String name;

  @Schema(description = DESCRIPTION_DESCRIPTION)
  private String description;

  @Schema(description = EXPRESSION_DESCRIPTION, example = EXPRESSION_EXAMPLE)
  @NotBlank
  private String expression;

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

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
    ContentSelectorApiCreateRequest that = (ContentSelectorApiCreateRequest) o;
    return Objects.equals(name, that.name) && Objects.equals(description, that.description) &&
        Objects.equals(expression, that.expression);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, description, expression);
  }
}
