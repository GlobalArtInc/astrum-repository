
package org.globalart.astrum.bootstrap.blobstore.file;

import jakarta.inject.Provider;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.file.FileBlobStore;

import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class FileBlobStoreConfiguration
{
  @Bean
  public Provider<BlobStore> fileBlobstoreProvider(final ObjectProvider<FileBlobStore> provider) {
    return new FileBlobStoreProvider(provider);
  }

  @Qualifier(FileBlobStore.TYPE)
  private static record FileBlobStoreProvider(ObjectProvider<FileBlobStore> provider)
      implements Provider<BlobStore>
  {
    @Override
    public BlobStore get() {
      return provider.getIfUnique();
    }
  }
}
