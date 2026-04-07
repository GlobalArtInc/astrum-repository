package org.globalart.astrum.repository.docker.internal.proxy;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.httpclient.AutoBlockConfiguration;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerFormat.NAME)
public class DockerAutoBlockConfiguration
    implements AutoBlockConfiguration
{
  @Override
  public boolean shouldBlock(final int statusCode) {
    return statusCode == 407 || statusCode >= 500;
  }
}
