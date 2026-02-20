
package org.globalart.astrum.repository.rest.internal.api;

import java.util.Collection;

public class RepositoryNugetXO
    extends RepositoryDetailXO
{
  private final String nugetVersion;

  private final Collection<String> memberNames;

  public RepositoryNugetXO(
      final String name,
      final String type,
      final String format,
      final String url,
      final RepositoryStatusXO status,
      final String nugetVersion,
      final Collection<String> memberNames)
  {
    super(name, type, format, url, status);
    this.nugetVersion = nugetVersion;
    this.memberNames = memberNames;
  }

  public String getNugetVersion() {
    return nugetVersion;
  }

  public Collection<String> getMemberNames() {
    return memberNames;
  }
}
