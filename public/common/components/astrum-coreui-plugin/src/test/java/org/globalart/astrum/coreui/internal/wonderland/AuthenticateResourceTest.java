
package org.globalart.astrum.coreui.internal.wonderland;

import java.util.Base64;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class AuthenticateResourceTest
{
  @Mock
  private AuthTicketService authTicketService;

  @Test
  void postReturnsTicketForAuthenticatedUser() {
    AuthenticateResource underTest = new AuthenticateResource(authTicketService);

    String username = "admin";
    String password = "password";
    String encodedUsername = Base64.getEncoder().encodeToString(username.getBytes());
    String encodedPassword = Base64.getEncoder().encodeToString(password.getBytes());

    AuthTokenXO token = new AuthTokenXO().withU(encodedUsername).withP(encodedPassword);
    when(authTicketService.createTicket(eq(username), anyString())).thenReturn("test-ticket");

    AuthTicketXO result = underTest.post(token);

    assertThat(result, is(notNullValue()));
    assertThat(result.getT(), is("test-ticket"));
  }

  @Test
  void postThrowsBadRequestWhenUsernameDoesNotMatchPrincipal() {
    AuthenticateResource underTest = new AuthenticateResource(authTicketService);

    String username = "different-user";
    String password = "password";
    String encodedUsername = Base64.getEncoder().encodeToString(username.getBytes());
    String encodedPassword = Base64.getEncoder().encodeToString(password.getBytes());

    AuthTokenXO token = new AuthTokenXO().withU(encodedUsername).withP(encodedPassword);

    WebApplicationException exception =
        assertThrows(WebApplicationException.class, () -> underTest.post(token));

    assertThat(exception.getResponse().getStatus(), is(Status.BAD_REQUEST.getStatusCode()));
  }

  @Test
  void postThrowsNullPointerExceptionWhenTokenIsNull() {
    AuthenticateResource underTest = new AuthenticateResource(authTicketService);
    assertThrows(NullPointerException.class, () -> underTest.post(null));
  }
}
