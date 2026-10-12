package org.globalart.astrum.repository.swift.internal.store;

import java.util.List;

import org.globalart.astrum.repository.content.store.ComponentDAO;

import org.apache.ibatis.annotations.Param;

public interface SwiftComponentDAO
    extends ComponentDAO
{
  @Override
  void extendSchema();

  void addPackageUrl(
      @Param("repositoryId") int repositoryId,
      @Param("componentId") int componentId,
      @Param("url") String url);

  void deletePackageUrls(@Param("componentId") int componentId);

  List<String> findPackageIdentifiers(@Param("repositoryId") int repositoryId, @Param("url") String url);
}
