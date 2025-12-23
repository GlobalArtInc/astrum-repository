
package org.globalart.astrum.security;

public class ErrorMessageUtil
{

  private static final String ERROR_MESSAGE_TEMPLATE = "ValidationErrorXO{id='*', message='%s'}";

  public static String getFormattedMessage(String message) {
    return String.format(ERROR_MESSAGE_TEMPLATE, message);
  }
}
