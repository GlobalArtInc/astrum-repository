
package org.globalart.astrum.internal.atlas;

import java.io.IOException;
import java.net.NetworkInterface;
import java.net.SocketException;
import java.nio.file.FileStore;
import java.nio.file.FileSystems;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumProperties;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.SystemInformationHelper;
import org.globalart.astrum.common.node.DeploymentAccess;
import org.globalart.astrum.common.node.NodeAccess;

import org.apache.commons.lang3.SystemUtils;
import org.junit.Assert;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.mockito.Mock;
import org.mockito.Mockito;

import static java.util.Collections.emptyMap;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.core.IsEqual.equalTo;
import static org.hamcrest.core.IsNot.not;
import static org.hamcrest.text.IsEmptyString.isEmptyString;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Unit tests for {@link SystemInformationGeneratorImpl}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class SystemInformationGeneratorImplTest
{

  public static final Map<String, Object> UNAVAILABLE = SystemInformationGeneratorImpl.UNAVAILABLE;

  @Rule
  public TemporaryFolder tempFolder = new TemporaryFolder();

  @Mock
  private ApplicationDirectories applicationDirectories;

  @Mock
  private ApplicationVersion applicationVersion;

  @Mock
  private NodeAccess nodeAccess;

  @Mock
  private DeploymentAccess deploymentAccess;

  @Mock
  private List<SystemInformationHelper> systemInformationHelpers;

  @Mock
  private AstrumProperties astrumProperties;

  @Before
  public void setup() throws IOException {
    when(applicationVersion.getVersion()).thenReturn("3.77.0-SNAPSHOT");
    when(applicationVersion.getBuildRevision()).thenReturn("123456");
    when(applicationVersion.getBuildTimestamp()).thenReturn("2021-01-01T00:00:00Z");
    when(applicationVersion.getEdition()).thenReturn("oss");
    when(nodeAccess.getId()).thenReturn("ABCDE");
    when(deploymentAccess.getId()).thenReturn("ABCDE");
    when(applicationDirectories.getInstallDirectory()).thenReturn(tempFolder.newFile());
    when(applicationDirectories.getWorkDirectory()).thenReturn(tempFolder.newFile());
    when(applicationDirectories.getTemporaryDirectory()).thenReturn(tempFolder.newFile());
    when(astrumProperties.get()).thenReturn(emptyMap());
  }

  /**
   * reportFileStores runs successfully using FileSystem.default
   */
  @Test
  public void reportFileStoresRunsSuccessfullyUsingFileSystemDefault() {
    SystemInformationGeneratorImpl generator = mockSystemInformationGenerator();

    Map<String, Object> data = new HashMap<>();
    for (FileStore fs : FileSystems.getDefault().getFileStores()) {
      data.put(fs.toString(), generator.reportFileStore(fs));
    }

    for (Map.Entry<String, Object> entry : data.entrySet()) {
      assertThat(entry.getKey(), not(isEmptyString()));
      assertThat(entry.getValue(), notNullValue());
    }
  }

  /**
   * reportFileStores handles IOException gracefully
   */
  @Test
  public void reportFileStoresHandlesIOExceptionGracefully() throws IOException {
    SystemInformationGeneratorImpl generator = mockSystemInformationGenerator();

    FileStore fs = Mockito.mock(FileStore.class);
    when(fs.toString()).thenReturn("description");
    when(fs.type()).thenReturn("brokenfstype");
    when(fs.name()).thenReturn("brokenfsname");
    when(fs.getTotalSpace()).thenThrow(new IOException("testing"));

    Map<String, Object> fsReport = generator.reportFileStore(fs);

    Assert.assertEquals(UNAVAILABLE, fsReport);
  }

  /**
   * reportNetwork runs successfully using NetworkInterface.networkInterfaces
   */
  @Test
  public void reportNetworkRunsSuccessfullyUsingNetworkInterfaceNetworkInterfaces() throws SocketException {
    SystemInformationGeneratorImpl generator = mockSystemInformationGenerator();

    List<NetworkInterface> networkInterfaces = Collections.list(NetworkInterface.getNetworkInterfaces());
    Map<String, Object> data = new HashMap<>();
    for (NetworkInterface ni : networkInterfaces) {
      data.put(ni.getName(), generator.reportNetworkInterface(ni));
    }

    for (Map.Entry<String, Object> entry : data.entrySet()) {
      assertThat(entry.getKey(), not(isEmptyString()));
      assertThat(entry.getValue(), notNullValue());
    }
  }

  /**
   * reportNetwork handles SocketException gracefully
   */
  @Test
  public void reportNetworkHandlesSocketExceptionGracefully() throws SocketException {
    SystemInformationGeneratorImpl generator = mockSystemInformationGenerator();
    NetworkInterface intf = Mockito.mock(NetworkInterface.class);
    when(intf.getDisplayName()).thenReturn("brokenintf");
    when(intf.supportsMulticast()).thenThrow(new SocketException("testing"));

    Map<String, Object> report = generator.reportNetworkInterface(intf);

    Assert.assertEquals(UNAVAILABLE, report);
  }

  private SystemInformationGeneratorImpl mockSystemInformationGenerator() {
    return new SystemInformationGeneratorImpl(
        applicationDirectories,
        astrumProperties,
        applicationVersion,
        nodeAccess,
        deploymentAccess,
        systemInformationHelpers);
  }

  /**
   * environment sensitive data is hidden
   */
  @Test
  public void environmentSensitiveDataIsHidden() throws IOException {
    SystemInformationGeneratorImpl generator = mockSystemInformationGenerator();
    // we need to exec some command to set up environment variables.
    String cmd = "mvn";
    if (SystemUtils.IS_OS_WINDOWS) {
      cmd = "mvn.cmd";
    }
    ProcessBuilder processBuilder = new ProcessBuilder(cmd, "version");
    processBuilder.environment().put("AZURE_CLIENT_SECRET", "azureSecretValue");
    processBuilder.environment().put("AZURE_TOKEN", "azureTokenValue");
    processBuilder.environment().put("MY_PASSWORD_FOR_ASTRUM", "admin123");
    processBuilder.start();

    Map<String, Object> report = generator.report();

    Map<String, String> systemEnvs = (Map<String, String>) report.get("system-environment");
    assertThat(systemEnvs.get("AZURE_CLIENT_SECRET"), not("azureSecretValue"));
    assertThat(systemEnvs.get("AZURE_TOKEN"), not("azureTokenValue"));
    assertThat(systemEnvs.get("MY_PASSWORD_FOR_ASTRUM"), not("admin123"));
  }

  /**
   * jvm variable sensitive data is hidden
   */
  @Test
  public void jvmVariableSensitiveDataIsHidden() {
    when(astrumProperties.get()).thenReturn(Map.of(
        "sun.java.command",
        "test.variable=1 -Dastrum.password=astrum -Dastrum.token=123456"));
    SystemInformationGeneratorImpl generator = new SystemInformationGeneratorImpl(
        applicationDirectories,
        astrumProperties,
        applicationVersion,
        nodeAccess,
        deploymentAccess,
        systemInformationHelpers);

    Map<String, Object> report = generator.report();

    Map<String, Object> astrumProps = (Map<String, Object>) report.get("astrum-properties");
    assertThat(astrumProps.get("sun.java.command").toString(),
        equalTo("test.variable=1 -Dastrum.password=**** -Dastrum.token=****"));
  }

  /**
   * INSTALL4J_ADD_VM_PARAMS sensitive data is hidden
   */
  @Test
  public void install4jAddVmParamsSensitiveDataIsHidden() {
    when(astrumProperties.get()).thenReturn(Map.of(
        "INSTALL4J_ADD_VM_PARAMS",
        "test.variable=1 -Dastrum.password=astrum -Dastrum.token=123456"));
    SystemInformationGeneratorImpl generator = new SystemInformationGeneratorImpl(
        applicationDirectories,
        astrumProperties,
        applicationVersion,
        nodeAccess,
        deploymentAccess,
        systemInformationHelpers);

    Map<String, Object> report = generator.report();

    Map<String, Object> astrumProps = (Map<String, Object>) report.get("astrum-properties");
    assertThat(astrumProps.get("INSTALL4J_ADD_VM_PARAMS").toString(),
        equalTo("test.variable=1 -Dastrum.password=**** -Dastrum.token=****"));
  }

}
