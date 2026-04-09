
package org.globalart.astrum.repository.helm.internal.security;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.globalart.astrum.repository.helm.internal.HelmFormat;

/**
 * Helm format security resource.
 */
@Component
public class HelmFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public HelmFormatSecurityContributor(@Qualifier(HelmFormat.NAME) final Format format) {
    super(format);
  }
}
