
package org.globalart.astrum.repository.rest;

public class NoQueryParamException
    extends RuntimeException
{
  @Override
  public String getMessage() {
    return "At least one query parameter is required";
  }
}
