
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import java.util.HashMap;
import java.util.Map;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import uk.org.webcompere.systemstubs.environment.EnvironmentVariables;
import uk.org.webcompere.systemstubs.jupiter.SystemStub;
import uk.org.webcompere.systemstubs.jupiter.SystemStubsExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumDirectoryConfiguration.BASEDIR_SYS_PROP;
import static org.globalart.astrum.common.app.FeatureFlags.*;

@ExtendWith(MockitoExtension.class)
@ExtendWith({SystemStubsExtension.class})
class AstrumPropertiesVerifierTest
{
  private static final String TRUE = Boolean.TRUE.toString();

  private static final String FALSE = Boolean.FALSE.toString();

  private final AstrumPropertiesVerifier astrumPropertiesVerifier = new AstrumPropertiesVerifier();

  private final Map<String, String> properties = new HashMap<>();

  @Mock
  private AstrumProperties astrumProperties;

  @SystemStub
  private EnvironmentVariables environmentVariables;

  @BeforeEach
  void setUp() {
    // Treat the astrumProperties like a simple map
    lenient().doAnswer(invocation -> properties.get(invocation.getArgument(0)))
        .when(astrumProperties)
        .getProperty(anyString());
    lenient().doAnswer(invocation -> properties.put(invocation.getArgument(0), invocation.getArgument(1)))
        .when(astrumProperties)
        .put(anyString(), anyString());
  }

  @AfterEach
  void tearDown() {
    properties.clear();
  }

  @Test
  void verifiesMissingRequiredProperties() {
    assertThrows(IllegalStateException.class, () -> astrumPropertiesVerifier.verify(astrumProperties));
  }

  @Test
  void testVerifyRequiresProperties() {
    mockRequiredProperties();

    astrumPropertiesVerifier.verify(astrumProperties);
  }

  @Test
  void testVerifyEnsureHACDisabled() {
    mockRequiredProperties();

    // missing HA-C property does not throw exception
    assertDoesNotThrow(() -> astrumPropertiesVerifier.verify(astrumProperties));

    astrumProperties.put("astrum.clustered", "false");
    assertDoesNotThrow(() -> astrumPropertiesVerifier.verify(astrumProperties));

    astrumProperties.put("astrum.clustered", "true");
    assertThrows(IllegalStateException.class, () -> astrumPropertiesVerifier.verify(astrumProperties));
  }

  @Test
  void testSelectDatastoreFeature_WhenClusteredEnabledOnEnvironment_SetsRelatedFeatures() {
    mockRequiredProperties();

    environmentVariables.set("DATASTORE_CLUSTERED_ENABLED", "true");

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(DATASTORE_CLUSTERED_ENABLED), is(TRUE));

    // datastore features
    assertThat(astrumProperties.getProperty(SQL_DISTRIBUTED_CACHE), is(TRUE));
    assertThat(astrumProperties.getProperty(DATASTORE_BLOBSTORE_METRICS), is(TRUE));

    // authentication features
    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(FALSE));
  }

  @Test
  void testSelectDatastoreFeature_WhenClusteredDisabledOnEnvironment_SetsRelatedFeatures() {
    mockRequiredProperties();
    environmentVariables.set("DATASTORE_CLUSTERED_ENABLED", FALSE);

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(CLUSTERED_ZERO_DOWNTIME_ENABLED), is(FALSE));
    assertThat(astrumProperties.getProperty(DATASTORE_CLUSTERED_ENABLED), is(FALSE));

    // set because DATASTORE_CLUSTERED_ENABLED is false and both SESSION_ENABLED and JWT_ENABLED are unset
    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(FALSE));

    // left unset
    assertThat(astrumProperties.getProperty(SQL_DISTRIBUTED_CACHE), nullValue());
    assertThat(astrumProperties.getProperty(DATASTORE_BLOBSTORE_METRICS), nullValue());
  }

  @Test
  void testSelectDatastoreFeature_AlwaysSetsDbFeatureAndDefaults() {
    mockRequiredProperties();

    when(astrumProperties.get(CHANGE_REPO_BLOBSTORE_TASK_ENABLED, "true")).thenReturn(TRUE);

    astrumPropertiesVerifier.verify(astrumProperties);

    verify(astrumProperties).get(CHANGE_REPO_BLOBSTORE_TASK_ENABLED, "true");

    assertThat(astrumProperties.getProperty(AstrumPropertiesVerifier.DB_FEATURE_PROPERTY_KEY),
        is("astrum-datastore-mybatis"));
    assertThat(astrumProperties.getProperty(CHANGE_REPO_BLOBSTORE_TASK_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty("astrum.quartz.jobstore.jdbc"), is(TRUE));
  }

  @Test
  void testSelectAuthenticationFeature_WhenJwtEnabled_OAuth2RemainsEnabled() {
    mockRequiredProperties();
    astrumProperties.put(JWT_ENABLED, TRUE);

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(ASTRUM_SECURITY_OAUTH2_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(FALSE));
  }

  @Test
  void testSelectAuthenticationFeature_WhenDatastoreClusteredEnabled_OAuth2RemainsEnabled() {
    mockRequiredProperties();
    astrumProperties.put(DATASTORE_CLUSTERED_ENABLED, TRUE);

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(ASTRUM_SECURITY_OAUTH2_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(FALSE));
  }

  @Test
  void testSelectAuthenticationFeature_WhenJwtDisabled_OAuth2IsDisabled() {
    mockRequiredProperties();
    astrumProperties.put(JWT_ENABLED, FALSE);

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(ASTRUM_SECURITY_OAUTH2_ENABLED), is(FALSE));
    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(FALSE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(TRUE));
  }

  @Test
  void testSelectAuthenticationFeature_WhenNothingIsSet_JwtIsTheDefault() {
    mockRequiredProperties();
    environmentVariables.set(DATASTORE_CLUSTERED_ENABLED, FALSE);

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(ASTRUM_SECURITY_OAUTH2_ENABLED), is(TRUE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(FALSE));
  }

  @Test
  void testSelectAuthenticationFeature_WhenSessionEnabled_JwtAndOAuth2AreDisabled() {
    mockRequiredProperties();
    astrumProperties.put(SESSION_ENABLED, TRUE);

    astrumPropertiesVerifier.verify(astrumProperties);

    assertThat(astrumProperties.getProperty(JWT_ENABLED), is(FALSE));
    assertThat(astrumProperties.getProperty(ASTRUM_SECURITY_OAUTH2_ENABLED), is(FALSE));
    assertThat(astrumProperties.getProperty(SESSION_ENABLED), is(TRUE));
  }

  public void mockRequiredProperties() {
    astrumProperties.put(BASEDIR_SYS_PROP, "/tmp/astrum");
    astrumProperties.put(AstrumDirectoryConfiguration.DATADIR_SYS_PROP, "/tmp/astrum/data");
    astrumProperties.put(AstrumPropertiesVerifier.DB_FEATURE_PROPERTY_KEY, "astrum-datastore-mybatis");
  }
}
