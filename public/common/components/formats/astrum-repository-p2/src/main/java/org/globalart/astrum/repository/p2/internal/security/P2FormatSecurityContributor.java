
package org.globalart.astrum.repository.p2.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class P2FormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public P2FormatSecurityContributor(@Qualifier(P2Format.NAME) final Format format) {
    super(format);
  }
}
