
package org.globalart.astrum.email;

import org.apache.commons.mail2.jakarta.Email;
import org.apache.commons.mail2.core.EmailException;

/**
 * Email manager.
 *
 * @since 3.0
 */
public interface EmailManager
{
  /**
   * Returns copy of current email configuration.
   */
  EmailConfiguration getConfiguration();

  /**
   * Installs new email configuration.
   */
  void setConfiguration(EmailConfiguration configuration, String password);

  /**
   * Send an email.
   */
  void send(Email mail) throws EmailException;

  /**
   * Send verification email to given address using the provided password.
   */
  void sendVerification(EmailConfiguration configuration, String password, String address) throws EmailException;

  /**
   * Send verification email to given address using the existing password.
   */
  void sendVerification(EmailConfiguration configuration, String address) throws EmailException;

  /**
   * Create a new and empty {@link EmailConfiguration}
   *
   * @since 3.20
   */
  EmailConfiguration newConfiguration();

  String constructMessage(String message);
}
