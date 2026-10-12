package org.globalart.astrum.repository.rubygems.internal;

public class RubygemsException
    extends RuntimeException
{
  private final int status;

  public RubygemsException(final int status, final String message) {
    super(message);
    this.status = status;
  }

  public int getStatus() {
    return status;
  }

  public static RubygemsException badRequest(final String message) {
    return new RubygemsException(400, message);
  }

  public static RubygemsException notFound(final String message) {
    return new RubygemsException(404, message);
  }
}
