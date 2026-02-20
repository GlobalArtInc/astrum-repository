
package org.globalart.astrum.repository.rest.internal.api;

import org.globalart.astrum.repository.Recipe;

public class RecipeXO
{
  private final String format;

  private final String type;

  public RecipeXO(final String format, final String type) {
    this.format = format;
    this.type = type;
  }

  public RecipeXO(Recipe recipe) {
    this.format = recipe.getFormat().getValue();
    this.type = recipe.getType().getValue();
  }

  public String getFormat() {
    return format;
  }

  public String getType() {
    return type;
  }
}
