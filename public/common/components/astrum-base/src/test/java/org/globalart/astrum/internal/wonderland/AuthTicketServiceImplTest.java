
package org.globalart.astrum.internal.wonderland;

import org.globalart.astrum.wonderland.AuthTicketCache;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link AuthTicketServiceImpl}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class AuthTicketServiceImplTest
{
  @Mock
  private AuthTicketGenerator authTicketGenerator;

  @Mock
  private AuthTicketCache authTicketCache;

  private AuthTicketServiceImpl underTest;

  @Before
  public void setup() {
    when(authTicketGenerator.generate()).thenReturn("ticket");
    when(authTicketCache.remove("user", "ticket", "AstrumAuthorizingRealm")).thenReturn(true);

    underTest = new AuthTicketServiceImpl(authTicketGenerator, authTicketCache);
  }

  @Test
  public void testRedeemTicket_sameUser() {
    String ticket = underTest.createTicket("user", "AstrumAuthorizingRealm");
    assertThat(underTest.redeemTicket("user", ticket, "AstrumAuthorizingRealm"), is(true));
  }

  @Test
  public void testRedeemTicket_differentUser() {
    String ticket = underTest.createTicket("user", "AstrumAuthorizingRealm");
    assertThat(underTest.redeemTicket("bad", ticket, "AstrumAuthorizingRealm"), is(false));
  }
}
