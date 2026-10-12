package org.globalart.astrum.repository.yum.internal.repodata;

import java.util.Map;

public enum MetadataType
{
  PRIMARY("primary", "metadata", RepodataXml.COMMON_NS, Map.of("rpm", RepodataXml.RPM_NS)),
  FILELISTS("filelists", "filelists", RepodataXml.FILELISTS_NS, Map.of()),
  OTHER("other", "otherdata", RepodataXml.OTHER_NS, Map.of());

  private final String type;

  private final String rootElement;

  private final String namespace;

  private final Map<String, String> prefixes;

  MetadataType(
      final String type,
      final String rootElement,
      final String namespace,
      final Map<String, String> prefixes)
  {
    this.type = type;
    this.rootElement = rootElement;
    this.namespace = namespace;
    this.prefixes = prefixes;
  }

  public String type() {
    return type;
  }

  public String rootElement() {
    return rootElement;
  }

  public String namespace() {
    return namespace;
  }

  public Map<String, String> prefixes() {
    return prefixes;
  }

  public String fileName(final String checksum) {
    return checksum + "-" + type + ".xml.gz";
  }
}
