
package org.globalart.astrum.repository.helm.internal.util;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.io.InputStream;


import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.internal.AssetKind;

import org.apache.commons.lang3.StringUtils;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.28
 */
@Component
public class HelmAttributeParser
{
  private TgzParser tgzParser;
  private YamlParser yamlParser;
  private ProvenanceParser provenanceParser;

  @Autowired
  public HelmAttributeParser(final TgzParser tgzParser,
                             final YamlParser yamlParser,
                             final ProvenanceParser provenanceParser) {
    this.tgzParser = checkNotNull(tgzParser);
    this.yamlParser = checkNotNull(yamlParser);
    this.provenanceParser = checkNotNull(provenanceParser);
  }

  public HelmAttributes getAttributes(final AssetKind assetKind, final InputStream inputStream) throws IOException {
    switch (assetKind) {
      case HELM_PACKAGE:
        return getAttributesFromInputStream(inputStream);
      case HELM_PROVENANCE:
        return getAttributesProvenanceFromInputStream(inputStream);
      default:
        return new HelmAttributes();
    }
  }

  private HelmAttributes getAttributesProvenanceFromInputStream(final InputStream inputStream) throws IOException {
    return provenanceParser.parse(inputStream);
  }

  private HelmAttributes getAttributesFromInputStream(final InputStream inputStream) throws IOException {
    try (InputStream is = tgzParser.getChartFromInputStream(inputStream)) {
      return new HelmAttributes(yamlParser.load(is));
    }
  }

  public static HelmAttributes validateAttributes(final HelmAttributes attributes) {
    if (StringUtils.isBlank(attributes.getName())) {
      throw new ValidationErrorsException("Metadata is missing the name attribute");
    }

    if (StringUtils.isBlank(attributes.getVersion())) {
      throw new ValidationErrorsException("Metadata is missing the version attribute");
    }

    return attributes;
  }
}
