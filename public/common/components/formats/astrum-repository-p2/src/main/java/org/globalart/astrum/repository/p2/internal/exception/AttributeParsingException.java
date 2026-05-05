
package org.globalart.astrum.repository.p2.internal.exception;

public class AttributeParsingException extends Exception
{
  private static final String EXCEPTION_MESSAGE = "Could not get attributes from jar";

  public AttributeParsingException() {
    super(EXCEPTION_MESSAGE);
  }

  public AttributeParsingException(final Exception ex) {
    super(EXCEPTION_MESSAGE + " due to " + ex.getMessage());
  }
}
