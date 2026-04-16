package org.globalart.astrum.repository.conda.internal.metadata;

import java.io.IOException;

public class InvalidCondaPackageException
    extends IOException
{
  private static final long serialVersionUID = 1L;

  public InvalidCondaPackageException(final String message) {
    super(message);
  }
}
