package org.globalart.astrum.repository.nuget.internal;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetFormat
    extends Format
{
  public static final String NAME = "nuget";

  public NugetFormat() {
    super(NAME);
  }
}
