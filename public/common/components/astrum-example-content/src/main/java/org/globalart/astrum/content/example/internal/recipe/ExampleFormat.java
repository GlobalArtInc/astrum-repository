
package org.globalart.astrum.content.example.internal.recipe;

import org.globalart.astrum.repository.Format;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Example repository format.
 *
 * @since 3.24
 */
@Component
@Qualifier(ExampleFormat.NAME)
public class ExampleFormat
    extends Format
{
  public static final String NAME = "example";

  public ExampleFormat() {
    super(NAME);
  }
}
