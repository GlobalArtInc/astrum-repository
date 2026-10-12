package org.globalart.astrum.repository.gitlfs.internal.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.security.RepositoryFormatSecurityContributor;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
public class GitLfsFormatSecurityContributor
    extends RepositoryFormatSecurityContributor
{
  @Autowired
  public GitLfsFormatSecurityContributor(@Qualifier(GitLfsFormat.NAME) final Format format) {
    super(format);
  }
}
