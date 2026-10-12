package org.globalart.astrum.repository.swift.internal;

public class SwiftProblem
    extends RuntimeException
{
  private final int status;

  public SwiftProblem(final int status, final String detail) {
    super(detail);
    this.status = status;
  }

  public int getStatus() {
    return status;
  }

  public static SwiftProblem badRequest(final String detail) {
    return new SwiftProblem(400, detail);
  }

  public static SwiftProblem notFound(final String detail) {
    return new SwiftProblem(404, detail);
  }

  public static SwiftProblem methodNotAllowed(final String detail) {
    return new SwiftProblem(405, detail);
  }

  public static SwiftProblem conflict(final String detail) {
    return new SwiftProblem(409, detail);
  }

  public static SwiftProblem payloadTooLarge(final String detail) {
    return new SwiftProblem(413, detail);
  }

  public static SwiftProblem unsupportedMediaType(final String detail) {
    return new SwiftProblem(415, detail);
  }

  public static SwiftProblem unprocessable(final String detail) {
    return new SwiftProblem(422, detail);
  }
}
