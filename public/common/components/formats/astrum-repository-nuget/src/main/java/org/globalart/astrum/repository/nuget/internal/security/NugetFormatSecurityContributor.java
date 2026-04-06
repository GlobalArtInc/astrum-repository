package org.globalart.astrum.repository.nuget.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class NugetFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public NugetFormatSecurityContributor(@Qualifier(NugetFormat.NAME) final Format format) {
    super(format);
  }
}
