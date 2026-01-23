
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;

import jakarta.inject.Provider;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class BlobStoreGroupConfiguration
{
  @Bean
  public Provider<BlobStore> groupBlobStoreProvider(final ObjectProvider<BlobStoreGroup> provider) {
    return new GroupBlobStoreProvider(provider);
  }

  @Qualifier(BlobStoreGroup.TYPE)
  private static record GroupBlobStoreProvider(ObjectProvider<BlobStoreGroup> provider)
      implements Provider<BlobStore>
  {
    @Override
    public BlobStore get() {
      return provider.getIfUnique();
    }
  }
}
