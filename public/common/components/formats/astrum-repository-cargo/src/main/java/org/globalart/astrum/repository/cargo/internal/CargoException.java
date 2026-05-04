package org.globalart.astrum.repository.cargo.internal;

public class CargoException
    extends RuntimeException
{
  private final int status;

  public CargoException(final int status, final String message) {
    super(message);
    this.status = status;
  }

  public int getStatus() {
    return status;
  }

  public static CargoException badRequest(final String message) {
    return new CargoException(400, message);
  }

  public static CargoException notFound(final String message) {
    return new CargoException(404, message);
  }

  public static CargoException conflict(final String message) {
    return new CargoException(409, message);
  }

  public static CargoException forbidden(final String message) {
    return new CargoException(403, message);
  }
}
