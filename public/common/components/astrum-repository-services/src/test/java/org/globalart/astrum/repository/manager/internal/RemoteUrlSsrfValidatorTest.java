
package org.globalart.astrum.repository.manager.internal;

import java.util.HashMap;
import java.util.Map;
import java.util.stream.Stream;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.ValidationException;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.validation.ConstraintViolationFactory;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class RemoteUrlSsrfValidatorTest
{
  @Mock
  private ConstraintViolationFactory constraintViolationFactory;

  @Mock
  private Configuration configuration;

  @Mock
  private AntiSsrfService antiSsrfService;

  private RemoteUrlSsrfValidator validator;

  @BeforeEach
  void setup() {
    validator = new RemoteUrlSsrfValidator(constraintViolationFactory, antiSsrfService);
  }

  @Test
  void shouldAllowPublicUrl() {
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> proxyAttributes = new HashMap<>();
    proxyAttributes.put("remoteUrl", "https://repo1.maven.org/maven2/");
    attributes.put("proxy", proxyAttributes);

    when(configuration.getAttributes()).thenReturn(attributes);

    ConstraintViolation<?> result = validator.validate(configuration);

    assertThat(result, is(nullValue()));
  }

  static Stream<Arguments> blockedUrlTestData() {
    return Stream.of(
        Arguments.of("http://localhost:8080/", "localhost", "loopback address"),
        Arguments.of("http://192.168.1.1/", "192.168.1.1", "private network address"),
        Arguments.of("http://169.254.169.254/latest/meta-data/", "169.254.169.254", "restricted address"));
  }

  @ParameterizedTest
  @MethodSource("blockedUrlTestData")
  void shouldBlockRestrictedUrls(String url, String host, String errorMessage) {
    String expectedMessage = "Proxy URL blocked: " + errorMessage;
    ConstraintViolation<?> mockViolation = mock(ConstraintViolation.class);
    when(mockViolation.getMessage()).thenReturn(expectedMessage);
    when(constraintViolationFactory.createViolation("proxy.remoteUrl", expectedMessage))
        .thenReturn((ConstraintViolation) mockViolation);

    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> proxyAttributes = new HashMap<>();
    proxyAttributes.put("remoteUrl", url);
    attributes.put("proxy", proxyAttributes);

    when(configuration.getAttributes()).thenReturn(attributes);
    doThrow(new ValidationException(errorMessage))
        .when(antiSsrfService)
        .validateHostWithoutCache(host);

    ConstraintViolation<?> result = validator.validate(configuration);

    assertThat(result, is(notNullValue()));
    assertThat(result.getMessage(), is(expectedMessage));
  }

  @Test
  void shouldAllowNullProxyConfiguration() {
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    when(configuration.getAttributes()).thenReturn(attributes);

    ConstraintViolation<?> result = validator.validate(configuration);

    assertThat(result, is(nullValue()));
  }

  @Test
  void shouldAllowNullRemoteUrl() {
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    Map<String, Object> proxyAttributes = new HashMap<>();
    proxyAttributes.put("remoteUrl", null);
    attributes.put("proxy", proxyAttributes);

    when(configuration.getAttributes()).thenReturn(attributes);

    ConstraintViolation<?> result = validator.validate(configuration);

    assertThat(result, is(nullValue()));
  }

}
