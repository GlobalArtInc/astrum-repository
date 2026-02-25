
package org.globalart.astrum.repository.search.sql;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.List;

import org.globalart.astrum.common.collect.NestedAttributesMap;

/**
 * A record used for persistence in SQL search
 */
public interface SearchRecord
{
  void addAliasComponentName(String aliasComponentName);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  default void addFormatFieldValue1(final String formatFieldValue1) {
    addFormatFieldValue1(formatFieldValue1, false);
  }

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue1(String formatFieldValue1, boolean preventTokenization);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue2(String formatFieldValue2);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue3(String formatFieldValue3);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  default void addFormatFieldValue4(final String formatFieldValues4) {
    addFormatFieldValue4(formatFieldValues4, false);
  }

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue4(String formatFieldValues4, boolean preventTokenization);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue5(String formatFieldValue5);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  default void addFormatFieldValue6(final String formatFieldValues6) {
    addFormatFieldValue6(formatFieldValues6, false);
  }

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue6(String formatFieldValues6, boolean preventTokenization);

  /**
   * Add an attribute which is specific to a format this may be called multiple times for assets within the same
   * {@link Component} if needed. This should be for a single attribute of Components/Assets, do not mix attributes
   * within a single format.
   */
  void addFormatFieldValue7(String formatFieldValues7);

  void addKeyword(String value);

  void addKeywords(List<String> values);

  /**
   * Add an md5 for an asset, this may be called multiple times for assets within the same {@link Component} if needed.
   */
  void addMd5(String md5);

  /**
   * Add a path for an asset, this may be called multiple times for assets within the same {@link Component} if needed.
   */
  void addPath(String path);

  /**
   * Add an sha1 for an asset, this may be called multiple times for assets within the same {@link Component} if needed.
   */
  void addSha1(String sha1);

  /**
   * Add an sha-256 for an asset, this may be called multiple times for assets within the same {@link Component} if
   * needed.
   */
  void addSha256(String sha256);

  /**
   * Add an sha512 for an asset, this may be called multiple times for assets within the same {@link Component} if
   * needed.
   */
  void addSha512(String sha512);

  /**
   * Add an uploader for an asset, this may be called multiple times for assets within the same {@link Component} if
   * needed.
   */
  void addUploader(String uploader);

  /**
   * Add an uploader's IP address for an asset, this may be called multiple times for assets within the same
   * {@link Component} if needed.
   */
  void addUploaderIp(String uploaderIp);

  NestedAttributesMap attributes();

  Collection<String> getAliasComponentNames();

  String getComponentKind();

  String getComponentName();

  Integer getEntityVersion();

  Collection<String> getFormatFieldValues1();

  Collection<String> getFormatFieldValues2();

  Collection<String> getFormatFieldValues3();

  Collection<String> getFormatFieldValues4();

  Collection<String> getFormatFieldValues5();

  Collection<String> getFormatFieldValues6();

  Collection<String> getFormatFieldValues7();

  Collection<String> getKeywords();

  SearchAssetRecord newAssetRecord();

  OffsetDateTime getLastModified();

  Collection<String> getMd5();

  String getNamespace();

  String getNormalisedVersion();

  Collection<String> getPaths();

  Collection<String> getSha1();

  Collection<String> getSha256();

  Collection<String> getSha512();

  Collection<String> getTags();

  Collection<String> getUploaderIps();

  Collection<String> getUploaders();

  String getVersion();

  @Override
  int hashCode();

  boolean isPrerelease();

  void setAttributes(NestedAttributesMap attributes);

  void setComponentKind(String componentKind);

  void setComponentName(String componentName);

  void setEntityVersion(Integer entityVersion);

  Integer getComponentId();

  void setFormat(String format);

  void setLastModified(OffsetDateTime lastModified);

  void setNamespace(String namespace);

  void addNamespaceNames(String namespace);

  Collection<String> getNamespaceNames();

  void setNormalisedVersion(String normalisedVersion);

  void setPrerelease(boolean prerelease);

  void setTags(Collection<String> values);

  void setVersion(String version);

  void addVersionNames(String version);

  Collection<String> getVersionNames();

  Collection<SearchAssetRecord> getSearchAssetRecords();

  void addSearchAssetRecord(SearchAssetRecord searchAssetRecord);
}
