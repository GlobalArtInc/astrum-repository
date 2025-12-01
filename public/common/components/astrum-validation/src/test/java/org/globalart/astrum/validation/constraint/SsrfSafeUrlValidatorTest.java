
package org.globalart.astrum.validation.constraint;

import java.net.URI;

import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import jakarta.validation.ConstraintValidatorContext;
import jakarta.validation.ValidationException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.verify;

/**
 * Tests for {@link SsrfSafeUrlValidator}.
 */
@ExtendWith(MockitoExtension.class)
public class SsrfSafeUrlValidatorTest
{
  @Mock
  private ConstraintValidatorContext context;

  @Mock
  private ConstraintValidatorContext.ConstraintViolationBuilder violationBuilder;

  @Mock
  private AntiSsrfService antiSsrfService;

  private SsrfSafeUrlValidator underTest;

  @BeforeEach
  public void setUp() {
    underTest = new SsrfSafeUrlValidator(antiSsrfService);

    lenient().when(context.buildConstraintViolationWithTemplate(anyString())).thenReturn(violationBuilder);
    lenient().when(violationBuilder.addConstraintViolation()).thenReturn(context);
  }

  @Test
  public void testNullUrlIsvalid() {
    assertThat(underTest.isValid(null, context), is(true));
  }

  @Test
  public void testBlankHostIsvalid() {
    URI url = URI.create("http:///path");
    assertThat(underTest.isValid(url, context), is(true));
  }

  @Test
  public void testPublicUrlIsValid() {
    URI url = URI.create("https://example.com/webhook");
    assertThat(underTest.isValid(url, context), is(true));
  }

  @Test
  public void testLocalhostUrlIsInvalid() {
    doThrow(new ValidationException("Host resolves to private/local network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("localhost");

    URI url = URI.create("http://localhost:8080/webhook");
    assertThat(underTest.isValid(url, context), is(false));
    verify(context).disableDefaultConstraintViolation();
    verify(context).buildConstraintViolationWithTemplate("Host resolves to private/local network address");
  }

  @Test
  public void testLoopbackIpIsInvalid() {
    doThrow(new ValidationException("Host resolves to private/local network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("127.0.0.1");

    URI url = URI.create("http://127.0.0.1/webhook");
    assertThat(underTest.isValid(url, context), is(false));
  }

  @Test
  public void testPrivateIpIsInvalid() {
    doThrow(new ValidationException("Host resolves to private/local network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("10.0.0.1");

    URI url = URI.create("http://10.0.0.1/webhook");
    assertThat(underTest.isValid(url, context), is(false));
  }

  @Test
  public void testUrlWithUsernameAndPassword() {
    URI url = URI.create("https://user:pass@example.com/webhook");
    assertThat(underTest.isValid(url, context), is(true));
  }

  @Test
  public void testUrlWithPort() {
    URI url = URI.create("https://example.com:8443/webhook");
    assertThat(underTest.isValid(url, context), is(true));
  }

  @Test
  public void testUrlWithPath() {
    URI url = URI.create("https://example.com/path/to/webhook");
    assertThat(underTest.isValid(url, context), is(true));
  }

  @Test
  public void testUrlWithQuery() {
    URI url = URI.create("https://example.com/webhook?token=abc123");
    assertThat(underTest.isValid(url, context), is(true));
  }
}
