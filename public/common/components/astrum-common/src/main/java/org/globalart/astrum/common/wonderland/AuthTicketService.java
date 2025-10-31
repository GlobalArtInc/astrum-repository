
package org.globalart.astrum.common.wonderland;

/**
 * Authentication ticket service.
 *
 * @since 2.7
 */
public interface AuthTicketService
{
  /**
   * Create a new authentication ticket.
   */
  String createTicket(String user, String realmName);

  /**
   * Create a new authentication ticket for the currently logged in user.
   */
  String createTicket();

  /**
   * Redeem an authentication ticket.
   *
   * @return {@code true} if the authentication ticket was redeemed, else {@code false} if the ticket is invalid.
   */
  boolean redeemTicket(String user, String ticket, String realmName);

  /**
   * Redeem an authentication ticket for the currently logged in user.
   *
   * @return {@code true} if the authentication ticket was redeemed, else {@code false} if the ticket is invalid.
   */
  boolean redeemTicket(String ticket);
}
