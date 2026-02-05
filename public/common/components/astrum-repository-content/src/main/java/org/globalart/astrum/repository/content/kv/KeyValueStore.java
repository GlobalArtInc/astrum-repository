
package org.globalart.astrum.repository.content.kv;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.property.SystemPropertiesHelper;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.store.ContentStoreSupport;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.ibatis.annotations.Param;

public class KeyValueStore<T extends KeyValueDAO>
    extends ContentStoreSupport<T>
{
  private static final int DELETE_BATCH_SIZE_DEFAULT =
      SystemPropertiesHelper.getInteger("astrum.content.deleteBatchSize", 1000);

  @Autowired
  public KeyValueStore(
      final DataSessionSupplier sessionSupplier,
      final String contentStoreName,
      final Class<T> daoClass)
  {
    super(sessionSupplier, contentStoreName, daoClass);
  }

  @Transactional
  public Optional<String> get(final int repositoryId, final String type, final String key) {
    return dao().get(repositoryId, type, key);
  }

  @Transactional
  public Optional<KeyValue> getKeyValue(final int repositoryId, final String type, final String key) {
    return dao().getKeyValue(repositoryId, type, key);
  }

  @Transactional
  public void set(final int repositoryId, final String type, final String key, final String value) {
    dao().set(repositoryId, type, key, value);
  }

  @Transactional
  public void remove(final int repositoryId, final String type, final String key) {
    dao().remove(repositoryId, type, key);
  }

  /*
   * Transactional is intentionally omitted
   */
  public void removeAll(final int repositoryId, @Nullable final String category) {
    int count;
    do {
      count = removeCategoryPage(repositoryId, category);
    }
    while (count > 0);
  }

  @Transactional
  protected int removeCategoryPage(final int repositoryId, @Nullable final String category) {
    return dao().removeAll(repositoryId, category, DELETE_BATCH_SIZE_DEFAULT);
  }

  /*
   * Transactional is intentionally omitted
   */
  public void removeRepository(final int repositoryId) {
    int count;
    do {
      count = removeRepositoryPage(repositoryId);
    }
    while (count > 0);
  }

  @Transactional
  protected int removeRepositoryPage(final int repositoryId) {
    return dao().removeRepository(repositoryId, DELETE_BATCH_SIZE_DEFAULT);
  }

  @Transactional
  public Continuation<KeyValue> browse(
      final int repositoryId,
      final String type,
      final int limit,
      final String continuationToken)
  {
    return dao().browse(repositoryId, type, limit, continuationToken);
  }

  @Transactional
  public List<String> browseCategories(@Param("repositoryId") final int repositoryId) {
    return dao().browseCategories(repositoryId);
  }

  /**
   * Browse all entries within the repository.
   */
  @Transactional
  public int count(
      final int repositoryId,
      @Nullable final String category)
  {
    return dao().count(repositoryId, category);
  }

  @Transactional
  public List<String> findCategories(final int repositoryId, final String key) {
    return dao().findCategories(repositoryId, key);
  }

  @Transactional
  public List<KeyValue> findByCategoryAndKeyLike(
      final int repositoryId,
      @Nullable final String category,
      final String keyLike)
  {
    return dao().findByCategoryAndKeyLike(repositoryId, category, keyLike);
  }
}
