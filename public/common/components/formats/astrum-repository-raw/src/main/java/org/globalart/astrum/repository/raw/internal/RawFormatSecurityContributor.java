
package org.globalart.astrum.repository.raw.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * RAW format security contributor.
 *
 * @since 3.0
 */
@Component
public class RawFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public RawFormatSecurityContributor(@Qualifier(RawFormat.NAME) final Format format) {
    super(format);
  }
}
