package org.globalart.astrum.repository.nuget.internal.metadata;

public class InvalidNugetPackageException
    extends RuntimeException
{
  public InvalidNugetPackageException(final String message) {
    super(message);
  }
}
