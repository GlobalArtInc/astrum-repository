package org.globalart.astrum.repository.docker.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class DockerFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public DockerFormatSecurityContributor(@Qualifier(DockerFormat.NAME) final Format format) {
    super(format);
  }
}
