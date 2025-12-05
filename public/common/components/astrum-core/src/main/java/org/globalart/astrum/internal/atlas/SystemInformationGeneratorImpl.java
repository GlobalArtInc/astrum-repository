
package org.globalart.astrum.internal.atlas;

import java.io.File;
import java.io.IOException;
import java.net.NetworkInterface;
import java.net.SocketException;
import java.nio.file.FileStore;
import java.nio.file.FileSystems;
import java.util.Collections;
import java.util.Date;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Properties;
import java.util.TimeZone;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.time.Iso8601Date;
import org.globalart.astrum.bootstrap.entrypoint.configuration.AstrumProperties;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.SystemInformationHelper;
import org.globalart.astrum.common.atlas.SystemInformationGenerator;
import org.globalart.astrum.common.node.DeploymentAccess;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.text.Strings2;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.text.Strings2.MASK;
import org.springframework.stereotype.Component;

/**
 * Default {@link SystemInformationGenerator}.
 *
 * @since 2.7
 */
@Component
public class SystemInformationGeneratorImpl
    implements SystemInformationGenerator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ApplicationDirectories applicationDirectories;

  private final ApplicationVersion applicationVersion;

  private final AstrumProperties astrumProperties;

  private final NodeAccess nodeAccess;

  private final DeploymentAccess deploymentAccess;

  private final Map<String, SystemInformationHelper> systemInformationHelpers;

  static final Map<String, Object> UNAVAILABLE = Map.of("unavailable", true);

  private static final List<String> SENSITIVE_FIELD_NAMES =
      List.of("password", "secret", "token", "sign", "auth", "cred", "key", "pass");

  private static final List<String> SENSITIVE_CREDENTIALS_KEYS =
      List.of("sun.java.command", "INSTALL4J_ADD_VM_PARAMS");

  @Autowired
  public SystemInformationGeneratorImpl(
      final ApplicationDirectories applicationDirectories,
      final AstrumProperties astrumProperties,
      final ApplicationVersion applicationVersion,
      final NodeAccess nodeAccess,
      final DeploymentAccess deploymentAccess,
      final List<SystemInformationHelper> systemInformationHelpersList)
  {
    this.applicationDirectories = checkNotNull(applicationDirectories);
    this.applicationVersion = checkNotNull(applicationVersion);
    this.astrumProperties = checkNotNull(astrumProperties);
    this.nodeAccess = checkNotNull(nodeAccess);
    this.deploymentAccess = checkNotNull(deploymentAccess);
    this.systemInformationHelpers = QualifierUtil.buildQualifierBeanMap(checkNotNull(systemInformationHelpersList));
  }

  @Override
  public Map<String, Object> report() {
    log.info("Generating system information report");

    Map<String, Object> sections = new HashMap<>();
    sections.put("system-time", reportTime());
    sections.put("system-properties", reportObfuscatedProperties(System.getProperties()));
    sections.put("system-environment", reportObfuscatedProperties(System.getenv()));
    sections.put("system-runtime", reportRuntime());
    sections.put("system-network", reportNetwork());
    sections.put("system-filestores", reportFileStores());
    sections.put("astrum-status", reportAstrumStatus());
    sections.put("astrum-node", reportAstrumNode());
    sections.put("astrum-properties", reportObfuscatedProperties(astrumProperties.get()));
    sections.put("astrum-configuration", reportAstrumConfiguration());

    // Merge additional system information helpers
    sections.putAll(systemInformationHelpers.entrySet()
        .stream()
        .collect(Collectors.toMap(Map.Entry::getKey, entry -> entry.getValue().getValue())));

    return sections;
  }

  private Map<String, Object> reportTime() {
    Date now = new Date();
    Map<String, Object> data = new HashMap<>();
    data.put("timezone", TimeZone.getDefault().getID());
    data.put("current", now.getTime());
    data.put("iso8601", Iso8601Date.format(now));
    return data;
  }

  private Map<String, Object> reportRuntime() {
    Runtime runtime = Runtime.getRuntime();
    Map<String, Object> data = new HashMap<>();
    data.put("availableProcessors", runtime.availableProcessors());
    data.put("freeMemory", runtime.freeMemory());
    data.put("totalMemory", runtime.totalMemory());
    data.put("maxMemory", runtime.maxMemory());
    data.put("threads", Thread.activeCount());
    return data;
  }

  private Map<String, Object> reportFileStores() {
    Map<String, Object> fileStores = new HashMap<>();
    int counter = 1;
    for (FileStore store : FileSystems.getDefault().getFileStores()) {
      String key = store.name();
      while (fileStores.containsKey(key)) {
        key = store.name() + "-" + counter++;
      }
      fileStores.put(key, reportFileStore(store));
    }
    return fileStores;
  }

  private Map<String, Object> reportNetwork() {
    try {
      return Collections.list(NetworkInterface.getNetworkInterfaces())
          .stream()
          .collect(Collectors.toMap(NetworkInterface::getName, this::reportNetworkInterface));
    }
    catch (SocketException e) {
      log.error("Could not add report to support zip for network interfaces", e);
      return UNAVAILABLE;
    }
  }

  private Map<String, Object> reportAstrumStatus() {
    Map<String, Object> data = new HashMap<>();
    data.put("version", applicationVersion.getVersion());
    data.put("edition", applicationVersion.getEdition());
    data.put("buildRevision", applicationVersion.getBuildRevision());
    data.put("buildTimestamp", applicationVersion.getBuildTimestamp());
    return data;
  }

  private Map<String, Object> reportAstrumNode() {
    Map<String, Object> data = new HashMap<>();
    data.put("node-id", nodeAccess.getId());
    data.put("deployment-id", deploymentAccess.getId());
    return data;
  }

  private Map<String, Object> reportAstrumConfiguration() {
    Map<String, Object> data = new HashMap<>();
    data.put("installDirectory", fileref(applicationDirectories.getInstallDirectory()));
    data.put("workingDirectory", fileref(applicationDirectories.getWorkDirectory()));
    data.put("temporaryDirectory", fileref(applicationDirectories.getTemporaryDirectory()));
    return data;
  }

  private String fileref(File file) {
    try {
      return file != null ? file.getCanonicalPath() : null;
    }
    catch (IOException e) {
      log.error("Could not get canonical path for file {}", file, e);
      return null;
    }
  }

  Map<String, Object> reportFileStore(FileStore store) {
    try {
      Map<String, Object> data = new HashMap<>();
      data.put("description", store.toString());
      data.put("type", store.type());
      data.put("totalSpace", store.getTotalSpace());
      data.put("usableSpace", store.getUsableSpace());
      data.put("unallocatedSpace", store.getUnallocatedSpace());
      data.put("readOnly", store.isReadOnly());
      return data;
    }
    catch (IOException e) {
      log.error("Could not add report to support zip for file store {}", store.name(), e);
      return UNAVAILABLE;
    }
  }

  Map<String, Object> reportNetworkInterface(NetworkInterface intf) {
    try {
      Map<String, Object> data = new HashMap<>();
      data.put("displayName", intf.getDisplayName());
      data.put("up", intf.isUp());
      data.put("virtual", intf.isVirtual());
      data.put("multicast", intf.supportsMulticast());
      data.put("loopback", intf.isLoopback());
      data.put("ptp", intf.isPointToPoint());
      data.put("mtu", intf.getMTU());
      data.put("addresses", Collections.list(intf.getInetAddresses())
          .stream()
          .map(Object::toString)
          .collect(Collectors.joining(",")));
      return data;
    }
    catch (SocketException e) {
      log.error("Could not add report to support zip for network interface {}", intf.getDisplayName(), e);
      return UNAVAILABLE;
    }
  }

  private Map<String, String> reportObfuscatedProperties(Properties properties) {
    Map<String, String> map = new HashMap<>();
    for (String key : properties.stringPropertyNames()) {
      map.put(key, properties.getProperty(key));
    }
    return reportObfuscatedProperties(map);
  }

  private Map<String, String> reportObfuscatedProperties(Map<String, String> properties) {
    return properties.entrySet()
        .stream()
        .filter(entry -> entry.getKey() != null && entry.getValue() != null)
        .collect(Collectors.toMap(
            Map.Entry::getKey,
            entry -> {
              String key = entry.getKey();
              String value = entry.getValue();
              for (String sensitiveName : SENSITIVE_FIELD_NAMES) {
                if (key.toLowerCase(Locale.US).contains(sensitiveName)) {
                  value = Strings2.mask(value);
                }
                if (SENSITIVE_CREDENTIALS_KEYS.contains(key) && value.contains(sensitiveName)) {
                  value = value.replaceAll(sensitiveName + "=\\S*", sensitiveName + "=" + MASK);
                }
              }
              return value;
            }));
  }
}
