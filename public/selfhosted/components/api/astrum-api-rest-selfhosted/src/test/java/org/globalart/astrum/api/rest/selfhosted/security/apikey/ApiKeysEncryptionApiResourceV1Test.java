
package org.globalart.astrum.api.rest.selfhosted.security.apikey;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.api.rest.selfhosted.security.apikey.model.ApiKeysReEncryptionRequestApiXO;
import org.globalart.astrum.crypto.apikey.ApiKeysReEncryptService;
import org.globalart.astrum.crypto.secrets.ReEncryptionNotSupportedException;
import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doThrow;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class ApiKeysEncryptionApiResourceV1Test
{
  @Mock
  private ApiKeysReEncryptService apiKeysReEncryptService;

  @InjectMocks
  private ApiKeysEncryptionApiResourceV1 underTest;

  @Test
  void testEncrypt() {
    ApiKeysReEncryptionRequestApiXO request =
        new ApiKeysReEncryptionRequestApiXO("algorithm", null, null);
    Response response = underTest.reEncrypt(request);
    assertThat(response.getStatus()).isEqualTo(Response.Status.ACCEPTED.getStatusCode());

    request = new ApiKeysReEncryptionRequestApiXO(null, null, null);
    response = underTest.reEncrypt(request);
    assertThat(response.getStatus()).isEqualTo(Response.Status.ACCEPTED.getStatusCode());
  }

  @Test
  void testEncrypt_ReEncryptionIsNotSupported() {
    doThrow(new ReEncryptionNotSupportedException("Re-encryption api key principals is not supported"))
        .when(apiKeysReEncryptService)
        .submitReEncryption(any(), any(), any());

    WebApplicationMessageException exception =
        assertThrows(WebApplicationMessageException.class,
            () -> underTest.reEncrypt(new ApiKeysReEncryptionRequestApiXO(null, null, null)));

    assertThat(exception.getResponse().getStatus()).isEqualTo(Status.BAD_REQUEST.getStatusCode());
    ValidationErrorXO error = (ValidationErrorXO) exception.getResponse().getEntity();
    assertThat(error.getMessage()).isEqualTo("Re-encryption api key principals is not supported");
  }

  @Test
  void testEncrypt_ReEncryptionIsRunning() {
    doThrow(new IllegalStateException("Re-encryption principals task is already running")).when(apiKeysReEncryptService)
        .submitReEncryption(any(), any(), any());

    WebApplicationMessageException exception =
        assertThrows(WebApplicationMessageException.class,
            () -> underTest.reEncrypt(new ApiKeysReEncryptionRequestApiXO(null, null, null)));

    assertThat(exception.getResponse().getStatus()).isEqualTo(Status.CONFLICT.getStatusCode());
    ValidationErrorXO error = (ValidationErrorXO) exception.getResponse().getEntity();
    assertThat(error.getMessage()).isEqualTo("Re-encryption principals task is already running");
  }

  @Test
  void testEncrypt_UnexpectedError() {
    doThrow(new RuntimeException("unexpected error")).when(apiKeysReEncryptService)
        .submitReEncryption(any(), any(), any());
    ApiKeysReEncryptionRequestApiXO request =
        new ApiKeysReEncryptionRequestApiXO(null, null, null);
    assertThrows(RuntimeException.class, () -> underTest.reEncrypt(request));
  }

  @Test
  void testEncryptWithCustomIterationsForDecryption() {
    ApiKeysReEncryptionRequestApiXO request =
        new ApiKeysReEncryptionRequestApiXO("PBKDF2WithHmacSHA1", 5000, "admin@example.com");
    Response response = underTest.reEncrypt(request);
    assertThat(response.getStatus()).isEqualTo(Response.Status.ACCEPTED.getStatusCode());
  }

  @Test
  void testEncryptWithAlgorithmAndIterations() {
    ApiKeysReEncryptionRequestApiXO request =
        new ApiKeysReEncryptionRequestApiXO("PBKDF2WithHmacSHA256", 10000, null);
    Response response = underTest.reEncrypt(request);
    assertThat(response.getStatus()).isEqualTo(Response.Status.ACCEPTED.getStatusCode());
  }
}
