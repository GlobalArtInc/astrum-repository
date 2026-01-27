
package org.globalart.astrum.blobstore.s3.internal;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import software.amazon.awssdk.auth.credentials.AwsBasicCredentials;
import software.amazon.awssdk.auth.credentials.AwsCredentials;
import software.amazon.awssdk.auth.credentials.AwsCredentialsProvider;

import static org.junit.jupiter.api.Assertions.assertSame;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
public class SharedS3CredentialsProviderTest
{
  @Mock(extraInterfaces = AutoCloseable.class)
  private AwsCredentialsProvider mockDelegate;

  @Test
  void resolveCredentials_delegatesToUnderlying() {
    AwsCredentials expected = AwsBasicCredentials.create("access", "secret");
    when(mockDelegate.resolveCredentials()).thenReturn(expected);

    SharedS3CredentialsProvider provider = new SharedS3CredentialsProvider(mockDelegate);

    AwsCredentials result = provider.resolveCredentials();

    assertSame(expected, result);
    verify(mockDelegate).resolveCredentials();
  }

  @Test
  void close_isNoOp_doesNotCloseDelegate() throws Exception {
    SharedS3CredentialsProvider provider = new SharedS3CredentialsProvider(mockDelegate);

    provider.close();

    // The mock also implements AutoCloseable via Mockito; verify close() was never called on it
    verify((AutoCloseable) mockDelegate, never()).close();
  }

  @Test
  void destroy_closesDelegate() throws Exception {
    SharedS3CredentialsProvider provider = new SharedS3CredentialsProvider(mockDelegate);

    provider.destroy();

    verify((AutoCloseable) mockDelegate).close();
  }
}
