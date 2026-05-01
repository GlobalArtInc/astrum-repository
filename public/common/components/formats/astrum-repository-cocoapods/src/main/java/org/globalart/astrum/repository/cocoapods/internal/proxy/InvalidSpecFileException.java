
package org.globalart.astrum.repository.cocoapods.internal.proxy;

public class InvalidSpecFileException
  extends Exception
{
  public InvalidSpecFileException(final String reason) {
    super(reason);
  }

  public InvalidSpecFileException(final String reason, Throwable throwable) {
    super(reason, throwable);
  }
}
