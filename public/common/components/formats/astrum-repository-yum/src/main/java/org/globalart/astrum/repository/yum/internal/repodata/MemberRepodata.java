package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;
import java.io.InputStream;

public interface MemberRepodata
{
  String name();

  Repomd repomd();

  InputStream open(RepomdData data) throws IOException;
}
