
package org.globalart.astrum.bootstrap.blobstore.s3;

import jakarta.inject.Provider;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.s3.internal.S3BlobStore;

import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class S3BlobStoreConfiguration
{
  @Bean
  public Provider<BlobStore> s3BlobstoreProvider(final ObjectProvider<S3BlobStore> provider) {
    return new S3Provider(provider);
  }

  @Qualifier(S3BlobStore.TYPE)
  private static record S3Provider(ObjectProvider<S3BlobStore> provider)
      implements Provider<BlobStore>
  {
    @Override
    public BlobStore get() {
      return provider.getIfUnique();
    }
  }
}
