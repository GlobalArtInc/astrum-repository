
package org.globalart.astrum.api.rest.selfhosted.security.secrets;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.api.rest.selfhosted.security.secrets.model.ReEncryptionRequestApiXO;
import org.globalart.astrum.crypto.secrets.MissingKeyException;
import org.globalart.astrum.crypto.secrets.ReEncryptService;
import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.WebApplicationMessageException;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.doThrow;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SecretsEncryptionApiResourceV1Tests
{
  @Mock
  private ReEncryptService reEncryptService;

  private SecretsEncryptionApiResourceV1 underTest;

  @Before
  public void setup() {
    underTest = new SecretsEncryptionApiResourceV1(reEncryptService);
  }

  @Test
  public void testTaskCreated() {
    ReEncryptionRequestApiXO request = new ReEncryptionRequestApiXO("test-key", null);
    Response response = underTest.reEncrypt(request);
    assertThat(response.getStatus()).isEqualTo(Response.Status.ACCEPTED.getStatusCode());
  }

  @Test
  public void testBadRequest() {
    doThrow(new MissingKeyException("key invalid")).when(reEncryptService).submitReEncryption(anyString(), anyString());
    ReEncryptionRequestApiXO request = new ReEncryptionRequestApiXO("test-key", "test@mail.com");
    WebApplicationMessageException exception =
        assertThrows(WebApplicationMessageException.class, () -> underTest.reEncrypt(request));

    assertThat(exception.getResponse().getStatus()).isEqualTo(Status.BAD_REQUEST.getStatusCode());
    ValidationErrorXO error = (ValidationErrorXO) exception.getResponse().getEntity();
    assertThat(error.getMessage()).isEqualTo("key invalid");
  }
}
