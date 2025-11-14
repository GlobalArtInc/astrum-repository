
package org.globalart.astrum.internal.wonderland;

import java.util.Optional;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.wonderland.AuthTicketCache;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default {@link AuthTicketService} implementation.
 *
 * @since 2.7
 */
@Component
public class AuthTicketServiceImpl
    implements AuthTicketService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AuthTicketGenerator authTicketGenerator;

  private final AuthTicketCache authTicketCache;

  @Autowired
  public AuthTicketServiceImpl(
      final AuthTicketGenerator authTicketGenerator,
      final AuthTicketCache authTicketCache)
  {
    this.authTicketGenerator = checkNotNull(authTicketGenerator);
    this.authTicketCache = checkNotNull(authTicketCache);
  }

  @Override
  public String createTicket(final String user, final String realmName) {
    String ticket = authTicketGenerator.generate();
    authTicketCache.add(user, ticket, realmName);
    return ticket;
  }

  @Override
  @Nullable
  public String createTicket() {
    Subject subject = SecurityUtils.getSubject();
    if (subject != null) {
      Optional<String> realmName = subject.getPrincipals().getRealmNames().stream().findFirst();
      return createTicket(subject.getPrincipal().toString(), realmName.orElse(null));
    }

    return null;
  }

  @Override
  public boolean redeemTicket(final String user, final String ticket, final String realmName) {
    checkNotNull(ticket);
    return authTicketCache.remove(user, ticket, realmName);
  }

  @Override
  public boolean redeemTicket(final String ticket) {
    Subject subject = SecurityUtils.getSubject();
    if (subject != null) {
      Optional<String> realmName = subject.getPrincipals().getRealmNames().stream().findFirst();
      return redeemTicket(subject.getPrincipal().toString(), ticket, realmName.orElse(null));
    }

    return false;
  }
}
