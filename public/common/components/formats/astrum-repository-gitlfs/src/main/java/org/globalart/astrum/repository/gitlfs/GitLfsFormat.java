package org.globalart.astrum.repository.gitlfs;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Git LFS repository format: large objects of git repositories, addressed by the SHA-256 of their content.
 */
@Component
@Qualifier(GitLfsFormat.NAME)
public class GitLfsFormat
    extends Format
{
  public static final String NAME = "gitlfs";

  public GitLfsFormat() {
    super(NAME);
  }
}
