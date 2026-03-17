
package org.globalart.astrum.content.example.internal.recipe;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Example format security contributor.
 *
 * @since 3.24
 */
@Component
public class ExampleFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public ExampleFormatSecurityContributor(@Qualifier(ExampleFormat.NAME) final Format format) {
    super(format);
  }
}
