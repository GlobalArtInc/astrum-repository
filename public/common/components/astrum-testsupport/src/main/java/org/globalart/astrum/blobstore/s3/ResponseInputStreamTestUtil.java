

package org.globalart.astrum.blobstore.s3;

import java.io.ByteArrayInputStream;
import java.io.InputStream;

import software.amazon.awssdk.core.ResponseInputStream;
import software.amazon.awssdk.services.s3.model.GetObjectResponse;

public class ResponseInputStreamTestUtil
{
  public static ResponseInputStream<GetObjectResponse> getResponseInputStream(String contents) {
    final InputStream inputStream = new ByteArrayInputStream(contents.getBytes());
    final GetObjectResponse response = GetObjectResponse.builder().build();
    return new ResponseInputStream<>(response, inputStream);
  }
}
