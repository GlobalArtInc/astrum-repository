
package org.globalart.astrum.repository.rest.api;

import java.util.Objects;

import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.DESCRIPTION_DESCRIPTION;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.EXPRESSION_DESCRIPTION;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.EXPRESSION_EXAMPLE;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.NAME_DESCRIPTION;
import static org.globalart.astrum.repository.rest.internal.resources.doc.ContentSelectorsResourceDoc.TYPE_DESCRIPTION;

/**
 * ContentSelector transfer object for REST APIs.
 */
@Schema
public class ContentSelectorApiResponse
    implements ValidatableContentSelectorRequest
{
  @Schema(description = NAME_DESCRIPTION)
  private String name;

  @Schema(description = TYPE_DESCRIPTION)
  private String type;

  @Schema(description = DESCRIPTION_DESCRIPTION)
  private String description;

  @Schema(description = EXPRESSION_DESCRIPTION, example = EXPRESSION_EXAMPLE)
  private String expression;

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
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
    ContentSelectorApiResponse that = (ContentSelectorApiResponse) o;
    return Objects.equals(name, that.name) && Objects.equals(type, that.type) &&
        Objects.equals(description, that.description) && Objects.equals(expression, that.expression);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, type, description, expression);
  }
}
