package org.globalart.astrum.repository.terraform.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class TerraformFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public TerraformFormatSecurityContributor(@Qualifier(TerraformFormat.NAME) final Format format) {
    super(format);
  }
}
