package org.globalart.astrum.repository.docker.internal.security;

import org.globalart.astrum.security.token.BearerToken;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerTokenRealm.NAME)
public class DockerBearerToken
    extends BearerToken
{
  public DockerBearerToken() {
    super(DockerTokenRealm.NAME);
  }
}
