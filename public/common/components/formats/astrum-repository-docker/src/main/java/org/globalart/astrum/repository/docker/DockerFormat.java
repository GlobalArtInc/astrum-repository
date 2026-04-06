package org.globalart.astrum.repository.docker;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerFormat.NAME)
public class DockerFormat
    extends Format
{
  public static final String NAME = "docker";

  public DockerFormat() {
    super(NAME);
  }
}
